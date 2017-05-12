#![allow(unknown_lints, wrong_self_convention, new_ret_no_self)]
use client::RedditClient;
use options::{ListingOptions, TimeFilter, LinkPost, SelfPost};
use structures::listing::Listing;
use responses::listing;
use traits::Created;
use errors::APIError;
use structures::listing::PostStream;

/// The `Subreddit` struct represents a subreddit and allows access to post listings
/// and data about the subreddit.
pub struct Subreddit<'a> {
    /// The name of the subreddit represented by this struct.
    pub name: String,
    client: &'a RedditClient,
}

impl<'a> PartialEq for Subreddit<'a> {
    fn eq(&self, other: &Subreddit) -> bool {
        self.name == other.name
    }
}

impl<'a> Subreddit<'a> {
    fn get_feed(&self, ty: &str, opts: ListingOptions) -> Result<Listing, APIError> {
        // We do not include the after/before parameter here so the pagination can adjust it later
        // on.
        let uri = format!("/r/{}/{}limit={}&raw_json=1", self.name, ty, opts.batch);
        let full_uri = format!("{}&{}", uri, opts.anchor);
        self.client
            .get_json::<listing::Listing>(&full_uri, false)
            .and_then(|res| Ok(Listing::new(self.client, uri, res.data)))
    }

    /// Creates a `Subreddit` from a client and the subreddit's name. Do not use this directly -
    /// use `Client.subreddit(NAME)` instead.
    pub fn create_new(client: &'a RedditClient, name: &str) -> Subreddit<'a> {
        Subreddit {
            client: client,
            name: name.to_owned(),
        }
    }

    /// Gets a listing of the hot feed for this subreddit. The first page may include some sticky
    /// posts in addtion to the expected posts.
    /// # Examples
    /// ```
    /// use rawr::client::RedditClient;
    /// use rawr::options::ListingOptions;
    /// use rawr::auth::AnonymousAuthenticator;
    /// let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
    /// let sub = client.subreddit("askreddit");
    /// let hot = sub.hot(ListingOptions::default());
    /// ```
    pub fn hot(&self, opts: ListingOptions) -> Result<Listing, APIError> {
        self.get_feed("hot?", opts)
    }

    /// Gets a `PostStream` of the new posts in the subreddit. The iterator will yield values
    /// forever, unless it is manually ended at some point. For tips, check the `PostStream` class.
    /// # Examples
    /// ```rust,no_run
    /// use rawr::prelude::*;
    /// let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
    /// let askreddit = client.subreddit("askreddit");
    /// for post in askreddit.new_stream() {
    ///
    /// }
    /// ```
    pub fn new_stream(self) -> PostStream<'a> {
        let url = format!("/r/{}/new?limit=5", self.name);
        PostStream::new(&self.client, url)
    }

    /// Gets a listing of the new feed for this subreddit.
    /// # Examples
    /// ```
    /// use rawr::client::RedditClient;
    /// use rawr::options::ListingOptions;
    /// use rawr::traits::Content;
    /// use rawr::auth::AnonymousAuthenticator;
    /// let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
    /// let sub = client.subreddit("programming");
    /// let mut new = sub.new(ListingOptions::default()).expect("Could not get new feed");
    /// assert_eq!(new.next().unwrap().subreddit().name, "programming");
    /// ```
    pub fn new(&self, opts: ListingOptions) -> Result<Listing, APIError> {
        self.get_feed("new?", opts)
    }

    /// Gets a listing of the rising feed for this subreddit. Usually much shorter than the other
    /// listings; may be empty.
    /// # Examples
    /// ```
    /// use rawr::client::RedditClient;
    /// use rawr::options::ListingOptions;
    /// use rawr::auth::AnonymousAuthenticator;
    /// let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
    /// let sub = client.subreddit("thanksobama");
    /// let rising = sub.rising(ListingOptions::default()).unwrap();
    /// assert_eq!(rising.count(), 0);
    /// ```
    pub fn rising(&self, opts: ListingOptions) -> Result<Listing, APIError> {
        self.get_feed("rising?", opts)
    }


    /// Gets a listing of the top feed for this subreddit. Also requires a time filter (
    /// `rawr::options::TimeFilter`) which is equivalent to the "links from: all time" dropdown
    /// on the website.
    /// # Examples
    /// ```
    /// use rawr::client::RedditClient;
    /// use rawr::options::{ListingOptions, TimeFilter};
    /// use rawr::auth::AnonymousAuthenticator;
    /// let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
    /// let sub = client.subreddit("thanksobama");
    /// let mut top = sub.top(ListingOptions::default(), TimeFilter::AllTime)
    ///     .expect("Request failed");
    /// assert_eq!(top.next().unwrap().title(), "Thanks Obama, for helping to protect the rights of over 9 million Americans.");
    /// ```
    pub fn top(&self, opts: ListingOptions, time: TimeFilter) -> Result<Listing, APIError> {
        let path = format!("top?{}&", time);
        self.get_feed(&path, opts)
    }

    /// Gets a listing of the controversial feed for this subreddit. Also requires a time filter (
    /// `rawr::options::TimeFilter`) which is equivalent to the "links from: all time" dropdown
    /// on the website.
    pub fn controversial(&self,
                         opts: ListingOptions,
                         time: TimeFilter)
                         -> Result<Listing, APIError> {
        let path = format!("controversial?{}&", time);
        self.get_feed(&path, opts)
    }

    /// Submits a link post to this subreddit using the specified parameters. If the link has
    /// already been posted, this will fail unless you specifically allow reposts.
    /// # Examples
    /// ## Allowing a link to be reposted
    /// ```
    /// use rawr::options::LinkPost;
    /// let post = LinkPost::new("rawr!", "http://example.com").resubmit();
    /// ```
    /// ## Submitting a post
    /// ```rust,no_run
    /// use rawr::auth::PasswordAuthenticator;
    /// use rawr::client::RedditClient;
    /// use rawr::options::LinkPost;
    /// let client = RedditClient::new("rawr", PasswordAuthenticator::new("a", "b", "c", "d"));
    /// let sub = client.subreddit("rust");
    /// let post = LinkPost::new("rawr!", "http://example.com");
    /// sub.submit_link(post).expect("Posting failed!");
    /// ```
    pub fn submit_link(&self, post: LinkPost) -> Result<(), APIError> {
        let body = format!("api_type=json&extension=json&kind=link&resubmit={}&sendreplies=true&\
                            sr={}&title={}&url={}",
                           post.resubmit,
                           self.name,
                           self.client.url_escape(post.title.to_owned()),
                           self.client.url_escape(post.link.to_owned()));
        self.client.post_success("/api/submit", &body, false)
    }

    /// Submits a text post (self post) to this subreddit using the specified title and body.
    /// # Examples
    /// ## Submitting a post
    /// ```rust,no_run
    /// use rawr::auth::PasswordAuthenticator;
    /// use rawr::client::RedditClient;
    /// use rawr::options::SelfPost;
    /// let client = RedditClient::new("rawr", PasswordAuthenticator::new("a", "b", "c", "d"));
    /// let sub = client.subreddit("rust");
    /// let post = SelfPost::new("I love rawr!", "You should download it *right now*!");
    /// sub.submit_text(post).expect("Posting failed!");
    /// ```
    pub fn submit_text(&self, post: SelfPost) -> Result<(), APIError> {
        let body = format!("api_type=json&extension=json&kind=self&sendreplies=true&sr={}\
                            &title={}&text={}",
                           self.name,
                           self.client.url_escape(post.title),
                           self.client.url_escape(post.text));
        self.client.post_success("/api/submit", &body, false)
    }

    /// Fetches information about a subreddit such as subscribers, active users and sidebar
    /// information.
    /// # Examples
    /// ```
    /// use rawr::client::RedditClient;
    /// use rawr::auth::AnonymousAuthenticator;
    /// let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
    /// let learn_programming = client.subreddit("learnprogramming").about()
    ///     .expect("Could not fetch 'about' data");
    /// assert_eq!(learn_programming.display_name(), "learnprogramming");
    /// ```
    pub fn about(&self) -> Result<SubredditAbout, APIError> {
        let url = format!("/r/{}/about?raw_json=1", self.name);
        self.client
            .get_json::<listing::SubredditAbout>(&url, false)
            .and_then(|res| Ok(SubredditAbout::new(res.data)))
    }

    /// Subscribes to the specified subredit, returning the result to show whether the API call
    /// succeeded or not.
    pub fn subscribe(&self) -> Result<(), APIError> {
        let body = format!("action=sub&sr_name={}", self.name);
        self.client.post_success("/api/subscribe", &body, false)
    }

    /// Unsubscribes to the specified subreddit, returning the result to show whether the API call
    /// succeeded or not.
    pub fn unsubscribe(&self) -> Result<(), APIError> {
        let body = format!("action=unsub&sr_name={}", self.name);
        self.client.post_success("/api/subscribe", &body, false)
    }
}

/// Information about a subreddit such as subscribers, sidebar text and active users.
pub struct SubredditAbout {
    data: listing::SubredditAboutData,
}

impl Created for SubredditAbout {
    fn created(&self) -> i64 {
        self.data.created
    }

    fn created_utc(&self) -> i64 {
        self.data.created_utc
    }
}

impl SubredditAbout {
    /// Creates a new `SubredditAbout` instance. Use `Subreddit.about()` instead to get
    /// information about a subreddit.
    pub fn new(data: listing::SubredditAboutData) -> SubredditAbout {
        SubredditAbout { data: data }
    }

    /// The number of subscribers to this subreddit.
    pub fn subscribers(&self) -> u64 {
        self.data.subscribers
    }

    /// The number of logged-in users who have viewed this subreddit in the last 15
    /// minutes.
    pub fn active_users(&self) -> u64 {
        self.data.accounts_active
    }

    /// Returns `true` if the subreddit is visible to the public (i.e. not invitation only)
    pub fn public(&self) -> bool {
        self.data.public_traffic
    }

    /// The display name of the subreddit, not including leading /r/
    pub fn display_name(&self) -> &str {
        &self.data.display_name
    }
}
