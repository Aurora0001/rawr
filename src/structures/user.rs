use structures::submission::FlairList;
use structures::listing::Listing;
use client::RedditClient;
use responses::FlairSelectorResponse;
use responses::user::{UserAbout as _UserAbout, UserAboutData};
use responses::listing::Listing as _Listing;
use traits::Created;
use errors::APIError;

/// Interface to a Reddit user, which can be used to access their karma and moderator status.
pub struct User<'a> {
    client: &'a RedditClient,
    /// The name of the user that this struct represents.
    pub name: String,
}

impl<'a> User<'a> {
    /// Internal method. Use `RedditClient.user(NAME)` instead.
    pub fn new(client: &'a RedditClient, name: &str) -> User<'a> {
        User {
            client: client,
            name: name.to_owned(),
        }
    }
    /// Gets information about this user.
    /// # Example
    /// ```
    /// use rawr::client::RedditClient;
    /// use rawr::auth::AnonymousAuthenticator;
    /// let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
    /// let user = client.user("Aurora0001").about().expect("User request failed");
    /// assert_eq!(user.id(), "eqyvc");
    /// ```
    pub fn about(self) -> Result<UserAbout, APIError> {
        UserAbout::new(self.client, self.name)
    }

    /// Gets a list of possible **user** flairs that can be added in this subreddit.
    ///
    /// User flairs apply on a per-subreddit basis, and some may not permit user flairs at all.
    /// If you do not have the privileges to set the flair for this user, you will receive
    /// a 403 error.
    pub fn flair_options(&self, subreddit: &str) -> Result<FlairList, APIError> {
        let body = format!("user={}", self.name);
        let url = format!("/r/{}/api/flairselector", subreddit);
        self.client
            .post_json::<FlairSelectorResponse>(&url, &body, false)
            .and_then(|res| Ok(FlairList::new(res.choices)))
    }

    /// Sets the flair for this user in the specified subreddit, using the specified template
    /// string. You can get the template string from `flair_options`; either:
    /// - use the returned `FlairList` and call the method `find_text` which will return the
    /// template ID of the flair with the specified text.
    /// - iterate through the `FlairList`, and get the `FlairChoice.flair_template_id` value.
    pub fn flair(&self, subreddit: &str, template: &str) -> Result<(), APIError> {
        let body = format!("api_type=json&user={}&flair_template_id={}",
                           self.name,
                           template);
        let url = format!("/r/{}/api/selectflair", subreddit);
        self.client.post_success(&url, &body, false)
    }

    /// Gets a list of *submissions* that the specified user has submitted. This endpoint is a
    /// listing and will continue yielding items until every item has been exhausted.
    /// # Examples
    /// ```
    /// use rawr::prelude::*;
    /// let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
    /// let user = client.user("Aurora0001");
    /// let submissions = user.submissions().expect("Could not fetch!");
    /// let mut i = 0;
    /// for submission in submissions.take(5) {
    ///     i += 1;
    /// }
    /// assert_eq!(i, 5);
    /// ```
    pub fn submissions(&self) -> Result<Listing, APIError> {
        let url = format!("/user/{}/submitted?raw_json=1", self.name);
        self.client
            .get_json::<_Listing>(&url, false)
            .and_then(|res| Ok(Listing::new(self.client, url, res.data)))
    }
    // TODO: implement comment, overview, gilded listings etc.
}

/// Information about a user from /r/username/about, such as karma and ID.
pub struct UserAbout {
    data: UserAboutData,
}

impl UserAbout {
    /// Internal method. Use `RedditClient.user(NAME).about()` instead.
    pub fn new(client: &RedditClient, name: String) -> Result<UserAbout, APIError> {
        let url = format!("/user/{}/about?raw_json=1", name);
        client.get_json::<_UserAbout>(&url, false)
            .and_then(|res| Ok(UserAbout { data: res.data }))
    }

    /// Gets the user's link karma (including self post karma as of July 19th, 2016).
    pub fn link_karma(&self) -> i64 {
        self.data.link_karma
    }

    /// Gets the user's comment karma.
    pub fn comment_karma(&self) -> i64 {
        self.data.comment_karma
    }

    /// Gets the user ID, not including kind, e.g. 'eqyvc'.
    pub fn id(&self) -> &str {
        &self.data.id
    }
}

impl Created for UserAbout {
    fn created(&self) -> i64 {
        self.data.created
    }

    fn created_utc(&self) -> i64 {
        self.data.created_utc
    }
}
