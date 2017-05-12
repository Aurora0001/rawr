use std::vec::IntoIter;
use std::thread;
use std::time::Duration;

use client::RedditClient;
use errors::APIError;
use options::ListingOptions;
use responses::listing;
use responses::messages::{Message as MessageData, MessageListing as _MessageListing};
use structures::user::User;
use structures::subreddit::Subreddit;
use structures::comment_list::CommentList;
use structures::comment::Comment;
use responses::comment::NewComment;
use traits::{Approvable, Created, Commentable, Content, Editable, PageListing};

/// A representation of a private message from Reddit.
pub struct Message<'a> {
    client: &'a RedditClient,
    data: MessageData,
}

impl<'a> Message<'a> {
    /// Internal method. Use `RedditClient.messages().inbox()` or `unread()` instead to get
    /// message listings and individual messages.
    pub fn new(client: &RedditClient, data: MessageData) -> Message {
        Message {
            client: client,
            data: data,
        }
    }

    /// Gets the full name (kind + id, e.g. 't1_a5bzp') of the parent of this submission.
    pub fn parent_id(&self) -> Option<String> {
        self.data.parent_id.to_owned()
    }

    /// Marks this message as read, so it will not show in the unread queue.
    pub fn mark_read(&self) -> Result<(), APIError> {
        let body = format!("id={}", self.name());
        self.client.post_success("/api/read_message", &body, false)
    }
}

impl<'a> Commentable<'a> for Message<'a> {
    fn reply_count(&self) -> u64 {
        panic!("The Reddit API does not appear to return the reply count to messages, so this \
                function is unavailable.");
    }

    fn replies(self) -> Result<CommentList<'a>, APIError> {
        panic!("The Reddit API does not seem to return replies to messages as expected, so this \
                function is unavailable.");
    }

    fn reply(&self, text: &str) -> Result<Comment, APIError> {
        let body = format!("api_type=json&text={}&thing_id={}",
                           self.client.url_escape(text.to_owned()),
                           self.name());
        self.client
            .post_json::<NewComment>("/api/comment", &body, false)
            .and_then(|res| {
                let data = res.json
                    .data
                    .things
                    .into_iter()
                    .next()
                    .ok_or_else(|| APIError::MissingField("things[0]"));
                Ok(Comment::new(self.client, try!(data).data))
            })
    }
}

impl<'a> Created for Message<'a> {
    fn created(&self) -> i64 {
        self.data.created
    }

    fn created_utc(&self) -> i64 {
        self.data.created_utc
    }
}

impl<'a> Content for Message<'a> {
    fn author(&self) -> User {
        let author = self.data.author.to_owned().unwrap_or(String::from("reddit"));
        User::new(self.client, &author)
    }

    fn author_flair_text(&self) -> Option<String> {
        panic!("The author flair text cannot be retrieved from messages.");
    }

    fn author_flair_css(&self) -> Option<String> {
        panic!("The author flair CSS cannot be retrieved from messages.");
    }

    fn subreddit(&self) -> Subreddit {
        let subreddit = self.data.subreddit.to_owned().unwrap_or(String::from("all"));
        Subreddit::create_new(self.client, &subreddit)
    }

    fn delete(self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        self.client.post_success("/api/del_msg", &body, false)
    }

    fn name(&self) -> &str {
        &self.data.name
    }
}

impl<'a> Approvable for Message<'a> {
    fn approve(&self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        self.client.post_success("/api/approve", &body, false)
    }

    fn remove(&self, spam: bool) -> Result<(), APIError> {
        let body = format!("id={}&spam={}", self.data.name, spam);
        self.client.post_success("/api/remove", &body, false)
    }

    fn ignore_reports(&self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        self.client.post_success("/api/ignore_reports", &body, false)
    }

    fn unignore_reports(&self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        self.client.post_success("/api/unignore_reports", &body, false)
    }
}

impl<'a> Editable for Message<'a> {
    fn edited(&self) -> bool {
        panic!("Reddit does not provide access to the edit time for messages.");
    }

    fn edited_time(&self) -> Option<i64> {
        panic!("Reddit does not provide access to the edit time for messages.");
    }

    fn edit(&mut self, text: &str) -> Result<(), APIError> {
        let body = format!("api_type=json&text={}&thing_id={}",
                           self.client.url_escape(text.to_owned()),
                           self.data.name);
        let res = self.client.post_success("/api/editusertext", &body, false);
        if let Ok(()) = res {
            // TODO: should we update body_html?
            self.data.body = text.to_owned();
        }
        res
    }

    fn body(&self) -> Option<String> {
        Some(self.data.body.to_owned())
    }

    fn body_html(&self) -> Option<String> {
        Some(self.data.body_html.to_owned())
    }
}

/// A helper struct which allows access to the inbox, unread messages and other message queues.
pub struct MessageInterface<'a> {
    client: &'a RedditClient,
}

impl<'a> MessageInterface<'a> {
    /// Internal method. Use `RedditClient.messages()` instead.
    pub fn new(client: &RedditClient) -> MessageInterface {
        MessageInterface { client: client }
    }

    /// Composes a private message to send to a user.
    /// # Examples
    /// ```rust,no_run
    /// use rawr::prelude::*;
    /// let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
    /// client.messages().compose("Aurora0001", "Test", "Hi!");
    // ```
    pub fn compose(&self, recipient: &str, subject: &str, body: &str) -> Result<(), APIError> {
        let body = format!("api_type=json&subject={}&text={}&to={}",
                           subject,
                           body,
                           recipient);
        self.client.post_success("/api/compose", &body, false)
    }

    /// Gets a list of all received messages that have not been deleted.
    pub fn inbox(&self, opts: ListingOptions) -> Result<MessageListing<'a>, APIError> {
        let uri = format!("/message/inbox?raw_json=1&limit={}", opts.batch);
        let full_uri = format!("{}&{}", uri, opts.anchor);
        self.client
            .get_json::<_MessageListing>(&full_uri, false)
            .and_then(|res| Ok(MessageListing::new(self.client, uri, res.data)))
    }

    /// Gets all messages that have **not** been marked as read.
    pub fn unread(&self, opts: ListingOptions) -> Result<MessageListing<'a>, APIError> {
        let uri = format!("/message/unread?raw_json=1&limit={}", opts.batch);
        let full_uri = format!("{}&{}", uri, opts.anchor);
        self.client
            .get_json::<_MessageListing>(&full_uri, false)
            .and_then(|res| Ok(MessageListing::new(self.client, uri, res.data)))
    }

    /// Gets a `MessageStream` of unread posts, marking each one as read after yielding it from
    /// the iterator. This can be useful to monitor /u/username mentions, replies to comments/posts
    /// and private messages.
    /// # Examples
    /// ```rust,no_run
    /// use rawr::prelude::*;
    /// let client = RedditClient::new("rawr", PasswordAuthenticator::new("a", "b", "c", "d"));
    /// for message in client.messages().unread_stream() {
    ///     println!("New message received.");
    /// }
    /// ```
    pub fn unread_stream(self) -> MessageStream<'a> {
        MessageStream::new(&self.client, String::from("/message/unread?limit=5"))
    }
}

// TODO: refactor Listing to cover this case too.

/// A listing of messages that will auto-paginate until all messages in the listing have been
/// exhausted.
pub struct MessageListing<'a> {
    client: &'a RedditClient,
    query_stem: String,
    data: listing::ListingData<MessageData>,
}

impl<'a> MessageListing<'a> {
    /// Internal method. Use `RedditClient.messages()` and request one of the message listings
    /// (e.g. `inbox(LISTING_OPTIONS)`).
    pub fn new(client: &RedditClient,
               query_stem: String,
               data: listing::ListingData<MessageData>)
               -> MessageListing {
        MessageListing {
            client: client,
            query_stem: query_stem,
            data: data,
        }
    }
}

impl<'a> PageListing for MessageListing<'a> {
    fn before(&self) -> Option<String> {
        self.data.before.to_owned()
    }

    fn after(&self) -> Option<String> {
        self.data.after.to_owned()
    }

    fn modhash(&self) -> Option<String> {
        self.data.modhash.to_owned()
    }
}

impl<'a> MessageListing<'a> {
    fn fetch_after(&mut self) -> Result<MessageListing<'a>, APIError> {
        match self.after() {
            Some(after_id) => {
                let url = format!("{}&after={}", self.query_stem, after_id);
                self.client
                    .get_json::<_MessageListing>(&url, false)
                    .and_then(|res| {
                        Ok(MessageListing::new(self.client, self.query_stem.to_owned(), res.data))
                    })
            }
            None => Err(APIError::ExhaustedListing),
        }
    }
}

impl<'a> Iterator for MessageListing<'a> {
    type Item = Message<'a>;
    fn next(&mut self) -> Option<Message<'a>> {
        if self.data.children.is_empty() {
            if self.after().is_none() {
                None
            } else {
                let mut new_listing = self.fetch_after().expect("After does not exist!");
                self.data.children.append(&mut new_listing.data.children);
                self.data.after = new_listing.data.after;
                self.next()
            }
        } else {
            let child = self.data.children.drain(..1).next().unwrap();
            Some(Message::new(self.client, child.data))
        }
    }
}

/// A stream of unread messages from oldest to newest. Before being yielded from this iterator,
/// each message will be marked as read (and will not show up in the unread queue again).
pub struct MessageStream<'a> {
    client: &'a RedditClient,
    current_iter: Option<IntoIter<Message<'a>>>,
    url: String,
}

impl<'a> MessageStream<'a> {
    /// Internal method. Use `Subreddit.new_stream()` instead.
    pub fn new(client: &'a RedditClient, url: String) -> MessageStream<'a> {
        MessageStream {
            current_iter: None,
            client: client,
            url: url,
        }
    }
}

impl<'a> Iterator for MessageStream<'a> {
    type Item = Message<'a>;
    fn next(&mut self) -> Option<Message<'a>> {
        if self.current_iter.is_some() {
            let mut iter = self.current_iter.take().unwrap();
            let next_iter = iter.next();
            if next_iter.is_some() {
                let res = next_iter.unwrap();
                loop {
                    // Loops until post is marked as read.
                    if res.mark_read().is_ok() {
                        thread::sleep(Duration::new(5, 0));
                        break;
                    }
                }
                self.current_iter = Some(iter);
                Some(res)
            } else {
                self.next()
            }
        } else {
            thread::sleep(Duration::new(5, 0));
            let req: Result<_MessageListing, APIError> = self.client.get_json(&self.url, false);
            let current_iter = if let Ok(res) = req {
                Some(res.data
                    .children
                    .into_iter()
                    .map(|i| Message::new(self.client, i.data))
                    .rev()
                    .collect::<Vec<Message<'a>>>()
                    .into_iter())
            } else {
                None
            };
            self.current_iter = current_iter;
            self.next()
        }
    }
}
