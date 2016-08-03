use std::vec::IntoIter;
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use std::collections::HashMap;
use client::RedditClient;
use structures::comment::Comment;
use responses::BasicThing;
use responses::listing;
use responses::comment::{Comment as _Comment, More};
use serde_json::{Value, from_value, from_str};
use std::io::Read;
use errors::APIError;
use traits::Content;

/// A list of comments that can be iterated through. Automatically fetches 'more' links when
/// necessary until all comments have been consumed, which can lead to pauses while loading
/// from the API.
/// # Examples
/// ```
/// use rawr::client::RedditClient;
/// use rawr::options::ListingOptions;
/// use rawr::traits::Commentable;
/// use rawr::auth::AnonymousAuthenticator;
/// let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
/// let announcements = client.subreddit("announcements");
/// let announcement = announcements.hot(ListingOptions::default())
///     .expect("Could not fetch announcements")
///     .next().unwrap();
/// // Usually less than 100 top-level comments are fetched at a time, but the CommentList
/// // fetches it for us!
/// let comments = announcement.replies().expect("Could not get comments").take(100);
/// ```
pub struct CommentList<'a> {
    client: &'a RedditClient,
    comments: Vec<Comment<'a>>,
    comment_hashes: HashMap<String, usize>,
    more: Vec<More>,
    link_id: String,
    parent: String,
}

impl<'a> CommentList<'a> {
    /// Creates a `CommentList` by storing all comments in the `CommentList.comments` list
    /// and all 'more' items in the `CommentList.more` list. Do not use this method - instead, use
    /// `Submission.replies()` or `Comment.replies()`.
    pub fn new(client: &'a RedditClient,
               link_id: String,
               parent: String,
               comment_list: Vec<BasicThing<Value>>)
               -> CommentList<'a> {
        let mut new_items = vec![];
        let mut new_mores = vec![];
        let mut hashes = HashMap::new();
        for item in comment_list {
            if item.kind == "t1" {
                let item = from_value::<_Comment>(item.data).unwrap();
                let comment = Comment::new(client, item);
                hashes.insert(comment.name().to_owned(), new_items.len());
                new_items.push(comment);
            } else if item.kind == "more" {
                let item = from_value::<More>(item.data).unwrap();
                new_mores.push(item);
            } else {
                unreachable!();
            }
        }

        CommentList {
            client: client,
            comments: new_items,
            more: new_mores,
            comment_hashes: hashes,
            link_id: link_id,
            parent: parent,
        }
    }

    /// Creates an empty listing, when there are no comments to show.
    pub fn empty(client: &'a RedditClient) -> CommentList<'a> {
        CommentList {
            client: client,
            link_id: String::new(),
            parent: String::new(),
            comments: vec![],
            more: vec![],
            comment_hashes: HashMap::new(),
        }
    }

    /// Adds a (pre-existing) comment to the reply list. This is an internal method, and does not
    /// actually post a comment, just adds one that has already been fetched.
    pub fn add_reply(&mut self, item: Comment<'a>) {
        self.comment_hashes.insert(item.name().to_owned(), self.comments.len());
        self.comments.push(item);
    }

    fn fetch_more(&mut self, more_item: More) -> CommentList<'a> {
        let params = format!("api_type=json&raw_json=1&link_id={}&children={}",
                             &self.link_id,
                             &more_item.children.join(","));
        let url = "/api/morechildren";
        self.client
            .ensure_authenticated(|| {
                let mut res = try!(self.client.post(url, false).body(&params).send());
                if res.status.is_success() {
                    // The "data" attribute is sometimes not present, so we have to unwrap it all
                    // manually
                    let mut result_str = String::new();
                    res.read_to_string(&mut result_str).unwrap();
                    let mut new_listing: Value = from_str(&result_str).unwrap();
                    let mut new_listing = new_listing.as_object_mut().unwrap();
                    let mut json = new_listing.remove("json").unwrap();
                    let mut json = json.as_object_mut().unwrap();
                    let data = json.remove("data");
                    if let Some(mut data) = data {
                        let mut things = data.as_object_mut().unwrap();
                        let things = things.remove("things").unwrap();
                        let things: Vec<BasicThing<Value>> = from_value(things).unwrap();
                        Ok(CommentList::new(self.client,
                                            self.link_id.to_owned(),
                                            self.parent.to_owned(),
                                            things))
                    } else {
                        Ok(CommentList::new(self.client,
                                            self.link_id.to_owned(),
                                            self.parent.to_owned(),
                                            vec![]))
                    }
                } else {
                    Err(APIError::HTTPError(res.status))
                }
            })
            .unwrap()
    }

    fn merge_more_comments(&mut self, list: CommentList<'a>) {
        let mut orphans: HashMap<String, Vec<Comment>> = HashMap::new();
        for item in list.comments {
            self.merge_comment(item, &mut orphans);
        }
    }

    fn merge_comment(&mut self,
                     mut item: Comment<'a>,
                     mut orphanage: &mut HashMap<String, Vec<Comment<'a>>>) {
        {
            if item.parent() == self.parent {
                self.add_reply(item);
                return;
            }

            let parent = self.comment_hashes.get(item.parent());
            if let Some(pos) = parent {
                self.comments[*pos].add_reply(item);
                return;
            }
        }
        {
            if let Some(orphaned) = orphanage.remove(item.parent()) {
                // The orphaned children will now be added to their parent.
                for orphan in orphaned {
                    item.add_reply(orphan);
                }
                self.merge_comment(item, &mut orphanage);
            } else {
                let name = item.name().to_owned();
                if let Some(mut list) = orphanage.remove(&name) {
                    list.push(item);
                    orphanage.insert(name, list);
                } else {
                    orphanage.insert(name, vec![item]);
                }
            }
        }
    }
}

impl<'a> Iterator for CommentList<'a> {
    type Item = Comment<'a>;
    fn next(&mut self) -> Option<Comment<'a>> {
        if self.comments.is_empty() {
            if self.more.is_empty() {
                None
            } else {
                // XXX: This code is hideous (see the fetch_more etc.) but it does work.
                // TODO: refactor (carefully!)
                let more_item = self.more.drain(..1).next().unwrap();
                let mut new_listing = self.fetch_more(more_item);
                self.more.append(&mut new_listing.more);
                // We've already consumed all of the items, so we can remove the mapping now.
                self.comment_hashes = HashMap::new();
                self.merge_more_comments(new_listing);
                self.next()
            }
        } else {
            // Draining breaks the comment_hashes map!
            let child = self.comments.drain(..1).next().unwrap();
            Some(child)
        }
    }
}

/// A stream of comments from oldest to newest that updates via polling every 5 seconds.
pub struct CommentStream<'a> {
    client: &'a RedditClient,
    set: VecDeque<String>,
    current_iter: Option<IntoIter<Comment<'a>>>,
    id: String,
    link_name: String,
}

impl<'a> CommentStream<'a> {
    /// Internal method. Use `Submission.reply_stream()` instead.
    pub fn new(client: &'a RedditClient, link_name: String, id: String) -> CommentStream<'a> {
        CommentStream {
            set: VecDeque::new(),
            current_iter: None,
            client: client,
            link_name: link_name,
            id: id,
        }
    }
}

impl<'a> Iterator for CommentStream<'a> {
    type Item = Comment<'a>;
    fn next(&mut self) -> Option<Comment<'a>> {
        if self.current_iter.is_some() {
            let mut iter = self.current_iter.take().unwrap();
            let next_iter = iter.next();
            if next_iter.is_some() {
                let res = next_iter.unwrap();
                let name = res.name().to_owned();
                // VecDeque.contains is not stable yet!
                let mut contains = false;
                for item in &self.set {
                    if item == &name {
                        contains = true;
                    }
                }
                if contains {
                    self.current_iter = Some(iter);
                    self.next()
                } else {
                    self.set.push_back(name);
                    if self.set.len() > 10 {
                        self.set.pop_front();
                    }
                    self.current_iter = Some(iter);
                    Some(res)
                }
            } else {
                self.next()
            }
        } else {
            thread::sleep(Duration::new(5, 0));
            let url = format!("/comments/{}?sort=new&raw_json=1", self.id);
            let req: Result<listing::CommentResponse, APIError> = self.client.get_json(&url, false);
            if let Ok(req) = req {
                let current_iter = CommentList::new(self.client,
                                                    self.link_name.to_owned(),
                                                    self.link_name.to_owned(),
                                                    req.1.data.children)
                    .take(5)
                    .collect::<Vec<Comment>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<Comment>>();
                self.current_iter = Some(current_iter.into_iter());
            }
            self.next()
        }
    }
}
