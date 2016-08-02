use std::vec::IntoIter;
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use responses::listing;
use client::RedditClient;
use structures::submission::Submission;
use traits::{Content, PageListing};
use errors::APIError;

/// A paginated listing of posts that can be iterated through. Posts are fetched lazily
/// until the listing is exhausted (similar to an infinite scroll of posts).
/// # Examples
/// ```rust,no_run
/// use rawr::client::RedditClient;
/// use rawr::options::ListingOptions;
/// use rawr::auth::AnonymousAuthenticator;
/// let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
/// let sub = client.subreddit("redditdev");
/// let mut hot = sub.hot(ListingOptions::default()).expect("Could not get hot posts");
/// for post in hot.take(500) {
///     // Do something with each post here
/// }
/// ```
/// # Gotchas
/// Be careful when looping directly over a listing - if you're iterating through a very long
/// listing, like /r/all/new, your code never stop!
///
/// Instead, prefer to use `Listing.take(n)` if possible, or require user input before continuing
/// to page.
///
/// ## Improving Performance
/// By default, rawr paginates using the same `limit` parameter as you
/// (`ListingOptions::default()` sets it to 25), so by default you can only fetch 25 posts
/// at a time. Create a `ListingOptions` object with a batch size of 100 to reduce the amount of
/// requests that are needed, like this:
///
/// ```
/// # use rawr::options::ListingOptions;
/// use rawr::options::ListingAnchor;
/// ListingOptions {
///     batch: 100,
///     anchor: ListingAnchor::None
/// };
/// ```
///
/// Keep in mind that if you only want 5 or 10 items, you might save bandwidth and get a quicker
/// response by using a smaller batch size (and the Reddit admins would love it if you didn't
/// waste bandwidth!)
pub struct Listing<'a> {
    client: &'a RedditClient,
    query_stem: String,
    data: listing::ListingData<listing::Submission>,
}

impl<'a> Listing<'a> {
    /// Internal method. Use other functions that return Listings, such as `Subreddit.hot()`.
    pub fn new(client: &RedditClient,
               query_stem: String,
               data: listing::ListingData<listing::Submission>)
               -> Listing {
        Listing {
            client: client,
            query_stem: query_stem,
            data: data,
        }
    }
}

impl<'a> PageListing for Listing<'a> {
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

impl<'a> Listing<'a> {
    fn fetch_after(&mut self) -> Result<Listing<'a>, APIError> {
        match self.after() {
            Some(after_id) => {
                let url = format!("{}&after={}", self.query_stem, after_id);
                self.client
                    .get_json::<listing::Listing>(&url, false)
                    .and_then(|res| {
                        Ok(Listing::new(self.client, self.query_stem.to_owned(), res.data))
                    })
            }
            None => Err(APIError::ExhaustedListing),
        }
    }
}

impl<'a> Iterator for Listing<'a> {
    type Item = Submission<'a>;
    fn next(&mut self) -> Option<Submission<'a>> {
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
            Some(Submission::new(self.client, child.data))
        }
    }
}

/// An infinite stream of posts which updates as posts are received. Stores a list of seen posts
/// so that each post is only seen once.
///
/// If the bot restarts, posts may be seen for a second time. To avoid this, you could send a
/// request to hide each thread (`Submission.hide()`) after you have processed it.
///
/// On extremely popular subreddits where more than 5 posts per second are being made, some may be
/// missed. If this is problematic for your use-case, file an issue on GitHub.
pub struct PostStream<'a> {
    client: &'a RedditClient,
    set: VecDeque<String>,
    current_iter: Option<IntoIter<Submission<'a>>>,
    url: String,
}

impl<'a> PostStream<'a> {
    /// Internal method. Use `Subreddit.new_stream()` instead.
    pub fn new(client: &'a RedditClient, url: String) -> PostStream<'a> {
        PostStream {
            set: VecDeque::new(),
            current_iter: None,
            client: client,
            url: url,
        }
    }
}

impl<'a> Iterator for PostStream<'a> {
    type Item = Submission<'a>;
    fn next(&mut self) -> Option<Submission<'a>> {
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
            let req: Result<listing::Listing, APIError> = self.client.get_json(&self.url, false);
            let current_iter = if let Ok(res) = req {
                Some(res.data
                    .children
                    .into_iter()
                    .map(|i| Submission::new(self.client, i.data))
                    .rev()
                    .collect::<Vec<Submission<'a>>>()
                    .into_iter())
            } else {
                None
            };
            self.current_iter = current_iter;
            self.next()
        }
    }
}
