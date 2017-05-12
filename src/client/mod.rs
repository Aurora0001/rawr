//! A client that represents one connection to the Reddit API. This can log in to one account
//! or remain anonymous, and performs all interactions with the Reddit API.
//! # Examples
//! ## Creating a RedditClient
//! When creating a `RedditClient`, you are only required to pass in a user agent string, which will
//! identify your client. The user agent should identify your program, but does not need to
//! be unique to this particular machine - you should use one user agent for each version of
//! your program. You **must** use a descriptive user agent when creating the client to comply
//! with Reddit API rules.
//!
//! The recommended format for user agent strings is `platform:program:version (by /u/yourname)`,
//! e.g. `linux:rawr:v0.0.1 (by /u/Aurora0001)`.
//!
//! You also need to pass in an *Authenticator*. `rawr` provides multiple authenticators that
//! use the different authentication flows provided by Reddit. To get started, you may just want
//! to browse anonymously. For this, `AnonymousAuthenticator` is provided, which can browse
//! reddit without any IDs or credentials.
//!
//! If you need logged-in privileges, you need to choose a different authenticator. For most
//! purposes, the appropriate authenticator will be `PasswordAuthenticator`. See the `auth` module
//! for examples of usage and benefits of this.
//!
//! ```
//! use rawr::client::RedditClient;
//! use rawr::auth::AnonymousAuthenticator;
//! let agent = "linux:rawr:v0.0.1 (by /u/Aurora0001)";
//! let client = RedditClient::new(agent, AnonymousAuthenticator::new());
//! ```

use std::sync::{Arc, Mutex, MutexGuard};
use std::io::Read;

use hyper::client::{Client, RequestBuilder};
use hyper::header::UserAgent;
use hyper::net::HttpsConnector;
use hyper::status::StatusCode::Unauthorized;
use hyper_native_tls::NativeTlsClient;

use serde_json::from_str;
use serde::de::DeserializeOwned;

use structures::subreddit::Subreddit;
use structures::user::User;
use structures::submission::LazySubmission;
use structures::messages::MessageInterface;
use auth::Authenticator;
use errors::APIError;

/// A client to connect to Reddit. See the module-level documentation for examples.
pub struct RedditClient {
    /// The internal HTTP client. You should not need to manually use this. If you do, file an
    /// issue saying why the API does not support your use-case, and we'll try to add it.
    pub client: Client,
    user_agent: String,
    authenticator: Arc<Mutex<Box<Authenticator + Send>>>,
    auto_logout: bool,
}


impl RedditClient {
    /// Creates an instance of the `RedditClient` using the provided user agent.
    pub fn new(user_agent: &str,
               authenticator: Arc<Mutex<Box<Authenticator + Send>>>)
               -> RedditClient {
        // Connection pooling is problematic if there are pauses/sleeps in the program, so we
        // choose to disable it by using a non-pooling connector.
        let ssl = NativeTlsClient::new().expect("Failed to acquire TLS client");
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);

        let this = RedditClient {
            client: client,
            user_agent: user_agent.to_owned(),
            authenticator: authenticator,
            auto_logout: true,
        };

        this.get_authenticator()
            .login(&this.client, &this.user_agent)
            .expect("Authentication failed. Did you use the correct username/password?");
        this
    }

    /// Disables the automatic logout that occurs when the client drops out of scope.
    /// In the case of OAuth, it will prevent your access token or refresh token from being
    /// revoked, though they may expire anyway.
    ///
    /// Although not necessary, it is good practice to revoke tokens when you're done with them.
    /// This will **not** affect the client ID or client secret.
    /// # Examples
    /// ```rust,no_run
    /// use rawr::client::RedditClient;
    /// use rawr::auth::PasswordAuthenticator;
    /// let mut client = RedditClient::new("rawr", PasswordAuthenticator::new("a", "b", "c", "d"));
    /// client.set_auto_logout(false); // Auto-logout disabled. Set to `true` to enable.
    /// ```
    pub fn set_auto_logout(&mut self, val: bool) {
        self.auto_logout = val;
    }

    /// Runs the lambda passed in. Refreshes the access token if it fails due to an HTTP 401
    /// Unauthorized error, then reruns the lambda. If the lambda fails twice, or fails due to
    /// a different error, the error is returned.
    pub fn ensure_authenticated<F, T>(&self, lambda: F) -> Result<T, APIError>
        where F: Fn() -> Result<T, APIError>
    {
        let res = lambda();
        match res {
            Err(APIError::HTTPError(Unauthorized)) => {
                try!(self.get_authenticator().refresh_token(&self.client, &self.user_agent));
                lambda()
            }
            _ => res,
        }
    }

    /// Gets a mutable reference to the authenticator using a `&RedditClient`. Mainly used
    /// in the `ensure_authenticated` method to update tokens if necessary.
    pub fn get_authenticator(&self) -> MutexGuard<Box<Authenticator + Send + 'static>> {
        self.authenticator.lock().unwrap()
    }

    /// Provides an interface to the specified subreddit which can be used to access
    /// subreddit-related API endpoints such as post listings.
    pub fn subreddit(&self, name: &str) -> Subreddit {
        Subreddit::create_new(self, &self.url_escape(name.to_owned()))
    }

    /// Gets the specified user in order to get user-related data such as the 'about' page.
    pub fn user(&self, name: &str) -> User {
        User::new(self, &self.url_escape(name.to_owned()))
    }

    /// Creates a full URL using the correct access point (API or OAuth) from the stem.
    pub fn build_url(&self,
                     dest: &str,
                     oauth_required: bool,
                     authenticator: &mut MutexGuard<Box<Authenticator + Send + 'static>>)
                     -> String {
        let oauth_supported = authenticator.oauth();
        let stem = if oauth_required || oauth_supported {
            // All endpoints support OAuth, but some do not support the regular endpoint. If we are
            // required to use it or support it, we will use it.
            assert!(oauth_supported,
                    "OAuth is required to use this endpoint, but your authenticator does not \
                     support it.");
            "https://oauth.reddit.com"
        } else {
            "https://api.reddit.com"
        };
        format!("{}{}", stem, dest)
    }

    /// Wrapper around the `get` function of `hyper::client::Client`, which sends a HTTP GET
    /// request. The correct user agent header is also sent using this function, which is necessary
    /// to prevent 403 errors.
    pub fn get(&self, dest: &str, oauth_required: bool) -> RequestBuilder {
        let mut authenticator = self.get_authenticator();
        let url = self.build_url(dest, oauth_required, &mut authenticator);
        let req = self.client.get(&url);
        let mut headers = authenticator.headers();
        headers.set(UserAgent(self.user_agent.to_owned()));
        req.headers(headers)
    }

    /// Sends a GET request with the specified parameters, and returns the resulting
    /// deserializeOwnedd object.
    pub fn get_json<T>(&self, dest: &str, oauth_required: bool) -> Result<T, APIError>
        where T: DeserializeOwned
    {
        self.ensure_authenticated(|| {
            let mut response = try!(self.get(dest, oauth_required).send());
            if response.status.is_success() {
                let mut buf = String::new();
                response.read_to_string(&mut buf).expect("Buffer read failed");
                let json: T = try!(from_str(&buf));
                Ok(json)
            } else {
                Err(APIError::HTTPError(response.status))
            }
        })
    }

    /// Wrapper around the `post` function of `hyper::client::Client`, which sends a HTTP POST
    /// request. The correct user agent header is also sent using this function, which is necessary
    /// to prevent 403 errors.
    pub fn post(&self, dest: &str, oauth_required: bool) -> RequestBuilder {
        let mut authenticator = self.get_authenticator();
        let url = self.build_url(dest, oauth_required, &mut authenticator);
        let req = self.client.post(&url);
        let mut headers = authenticator.headers();
        headers.set(UserAgent(self.user_agent.to_owned()));
        req.headers(headers)
    }

    /// Sends a post request with the specified parameters, and converts the resulting JSON
    /// into a deserializeOwnedd object.
    pub fn post_json<T>(&self, dest: &str, body: &str, oauth_required: bool) -> Result<T, APIError>
        where T: DeserializeOwned
    {
        self.ensure_authenticated(|| {
            let mut response = try!(self.post(dest, oauth_required).body(body).send());
            if response.status.is_success() {
                let mut buf = String::new();
                response.read_to_string(&mut buf).expect("Buffer read failed");
                let json: T = try!(from_str(&buf));
                Ok(json)
            } else {
                Err(APIError::HTTPError(response.status))
            }
        })
    }

    /// Sends a post request with the specified parameters, and ensures that the response
    /// has a success header (HTTP 2xx).
    pub fn post_success(&self,
                        dest: &str,
                        body: &str,
                        oauth_required: bool)
                        -> Result<(), APIError> {
        self.ensure_authenticated(|| {
            let response = try!(self.post(dest, oauth_required).body(body).send());
            if response.status.is_success() {
                Ok(())
            } else {
                Err(APIError::HTTPError(response.status))
            }
        })
    }

    /// URL encodes the specified string so that it can be sent in GET and POST requests.
    ///
    /// This is only done when data is being sent that isn't from the API (we assume that API
    /// data is safe)
    /// # Examples
    /// ```
    /// # use rawr::client::RedditClient;
    /// # use rawr::auth::AnonymousAuthenticator;
    /// # let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
    /// assert_eq!(client.url_escape(String::from("test&co")), String::from("test%26co"));
    /// assert_eq!(client.url_escape(String::from("👍")), String::from("%F0%9F%91%8D"));
    /// assert_eq!(client.url_escape(String::from("\n")), String::from("%0A"))
    /// ```
    pub fn url_escape(&self, item: String) -> String {
        let mut res = String::new();
        for character in item.chars() {
            match character {
                ' ' => res.push('+'),
                '*' | '-' | '.' | '0'...'9' | 'A'...'Z' | '_' | 'a'...'z' => res.push(character),
                _ => {
                    for val in character.to_string().as_bytes() {
                        res = res + &format!("%{:02X}", val);
                    }
                }
            }
        }
        res
    }

    /// Gets a `LazySubmission` object which can be used to access the information/comments of a
    /// specified post. The **full** name of the item should be used.
    /// # Examples
    /// ```
    /// use rawr::prelude::*;
    /// let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
    /// let post = client.get_by_id("t3_4uule8").get().expect("Could not get post.");
    /// assert_eq!(post.title(), "[C#] Abstract vs Interface");
    /// ```
    pub fn get_by_id(&self, id: &str) -> LazySubmission {
        LazySubmission::new(self, &self.url_escape(id.to_owned()))
    }

    /// Gets a `MessageInterface` object which allows access to the message listings (e.g. `inbox`,
    /// `unread`, etc.)
    /// # Examples
    /// ```rust,no_run
    /// use rawr::prelude::*;
    /// let client = RedditClient::new("rawr", PasswordAuthenticator::new("a", "b", "c", "d"));
    /// let messages = client.messages();
    /// for message in messages.unread(ListingOptions::default()) {
    ///
    /// }
    /// ```
    pub fn messages(&self) -> MessageInterface {
        MessageInterface::new(self)
    }
}

impl Drop for RedditClient {
    fn drop(&mut self) {
        if self.auto_logout {
            self.get_authenticator().logout(&self.client, &self.user_agent).unwrap();
        }
    }
}
