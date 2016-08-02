//! Whenever you create a `RedditClient`, you need to provide an `Authenticator` that can
//! log in for you and send credentials with every request. Reddit's API provides multiple
//! ways to authenticate, and it's important that you use the correct one for your application
//! so that you only take passwords where necessary.
//! # OAuth Or Not?
//! In effect, Reddit's API is split into two parts: a deprecated API that uses cookies for
//! authentication, and an OAuth API that is recommended for new applications. Clients that use
//! OAuth-based authenticators have a higher rate limit (60/min for OAuth, 30/min for cookie), so
//! it may be preferable to use OAuth if larger batches of data are being processed from the API.
//! # Authenticator Summary
//! - `AnonymousAuthenticator` - uses the legacy API (so it has a lower rate limit) but requires
//! no credentials at all. Choose this if you just want to **browse the API without registering**.
//! - `PasswordAuthenticator` - uses the OAuth API (so higher rate limits), but requires a
//! registered account and registration on the 'apps' page (see below). Choose this for **bots**
//! or scripts that use lots of data.
//!
//! TODO: Add authenticators for the other flows and document them.
//!
//! # Registering Your App (for OAuth-based authenticators)
//! **Note: this does not apply to `AnonymousAuthenticator`**.
//!
//! In order to register to use OAuth, you need to login on your **bot account** and create an
//! 'app'. All you have to do is:
//!
//! 1. Go to your [app preferences](https://www.reddit.com/prefs/apps)
//! 2. Click 'create app' (or 'create another app', if you've already created one)
//! 3. Enter a name for your bot.
//! 4. Leave the description and about URL blank (unless you want to fill these in!)
//! 5. Choose the correct app type. If you want to use `PasswordAuthenticator`, choose **script**.
//! 6. Set the redirect URL to 'http://www.example.com/rawr' - this will not be used.
//! 7. Click 'create app'.
//!
//! You'll probably be able to see something like [this](http://bit.ly/29PR8XN) now. If so, you've
//! successfully created your app. Don't close it yet, because we need to get some 'secrets'
//! from that page.
//! # OAuth Secrets
//! In addition to your username and password, `PasswordAuthenticator` requires a client ID and
//! client secret token. The only way to get this is by registering an app. If you've followed
//! the steps above, you're already in the right place to follow the next steps.
//!
//! On [this](http://bit.ly/29PR8XN) page, the client ID is the random string below 'personal use
//! script' and the client secret is the underlined string. Store both of these somewhere safe,
//! and treat the client secret like a password - it **must** not be shared with anyone!
//!
//! You're now ready to create a `PasswordAuthenticator`. Ensure you provide all parameters in the
//! correct order:
//!
//! ```rust,ignore
//! # use rawr::auth::PasswordAuthenticator;
//! PasswordAuthenticator::new(CLIENT_ID, CLIENT_SECRET, USERNAME, PASSWORD);
//! ```

#![allow(unknown_lints, doc_markdown)]

use std::sync::{Arc, Mutex};
use hyper;
use hyper::header::{Headers, Authorization, Basic, Bearer, UserAgent};
use std::io::Read;
use serde_json;
use responses::auth::TokenResponse;
use hyper::client::Client;
use errors::APIError;

/// Trait for any method of authenticating with the Reddit API.
pub trait Authenticator {
    /// Logs in and fetches relevant tokens.
    fn login(&mut self, client: &Client, user_agent: &str) -> Result<(), APIError>;
    /// Called if a token expiration error occurs.
    fn refresh_token(&mut self, client: &Client, user_agent: &str) -> Result<(), APIError> {
        self.login(client, user_agent)
    }
    /// Logs out and invalidates tokens if applicable.
    fn logout(&mut self, client: &Client, user_agent: &str) -> Result<(), APIError>;
    /// A list of OAuth scopes that this `Authenticator` can access. Currently, the result of this
    /// is not used, but the correct scopes should be returned. If all scopes can be accessed,
    /// this is signified by a vec!["*"]. If it is read-only, the result is vec!["read"].
    fn scopes(&self) -> Vec<String>;
    /// Returns the headers needed to authenticate. Must be done **after** `login()`.
    fn headers(&self) -> Headers;
    /// `true` if this authentication method requires the OAuth API.
    fn oauth(&self) -> bool;
}

/// An anonymous login authenticator.
pub struct AnonymousAuthenticator;

impl Authenticator for AnonymousAuthenticator {
    #[allow(unused_variables)]
    fn login(&mut self, client: &Client, user_agent: &str) -> Result<(), APIError> {
        // Don't log in, because we're anonymous!
        Ok(())
    }

    #[allow(unused_variables)]
    fn logout(&mut self, client: &Client, user_agent: &str) -> Result<(), APIError> {
        // Can't log out if we're not logged in.
        Ok(())
    }

    fn scopes(&self) -> Vec<String> {
        vec![String::from("read")]
    }

    fn headers(&self) -> Headers {
        Headers::new()
    }

    fn oauth(&self) -> bool {
        false
    }
}

impl AnonymousAuthenticator {
    /// Creates a new `AnonymousAuthenticator`. See the module-level documentation for the purpose
    /// of `AnonymousAuthenticator`.
    /// # Examples
    /// ```
    /// use rawr::auth::AnonymousAuthenticator;
    /// AnonymousAuthenticator::new();
    /// ```
    pub fn new() -> Arc<Mutex<Box<Authenticator + Send>>> {
        Arc::new(Mutex::new(Box::new(AnonymousAuthenticator {})))
    }
}

/// Authenticates using a username and password with OAuth. See the module-level documentation for
/// usage.
pub struct PasswordAuthenticator {
    access_token: Option<String>,
    client_id: String,
    client_secret: String,
    username: String,
    password: String,
}

impl Authenticator for PasswordAuthenticator {
    fn login(&mut self, client: &Client, user_agent: &str) -> Result<(), APIError> {
        let url = "https://www.reddit.com/api/v1/access_token";
        let body = format!("grant_type=password&username={}&password={}",
                           &self.username,
                           &self.password);
        let access_req = client.post(url)
            .header(Authorization(Basic {
                username: self.client_id.to_owned(),
                password: Some(self.client_secret.to_owned()),
            }))
            .header(UserAgent(user_agent.to_owned()))
            .body(&body);

        let mut result = access_req.send().unwrap();

        if result.status != hyper::Ok {
            Err(APIError::HTTPError(result.status))
        } else {
            let mut buf = String::new();
            result.read_to_string(&mut buf).unwrap();
            let token_response: TokenResponse = serde_json::from_str(&buf).unwrap();
            self.access_token = Some(token_response.access_token);
            Ok(())
        }
    }

    fn logout(&mut self, client: &Client, user_agent: &str) -> Result<(), APIError> {
        let url = "https://www.reddit.com/api/v1/revoke_token";
        let body = format!("token={}", &self.access_token.to_owned().unwrap());
        let req = client.post(url)
            .header(Authorization(Basic {
                username: self.client_id.to_owned(),
                password: Some(self.client_secret.to_owned()),
            }))
            .header(UserAgent(user_agent.to_owned()))
            .body(&body);
        let res = req.send().unwrap();
        if !res.status.is_success() {
            Err(APIError::HTTPError(res.status))
        } else {
            Ok(())
        }
    }

    fn scopes(&self) -> Vec<String> {
        vec![String::from("*")]
    }

    fn headers(&self) -> Headers {
        let mut headers = Headers::new();
        if let Some(ref token) = self.access_token {
            headers.set(Authorization(Bearer { token: token.to_owned() }));
        }
        headers
    }

    fn oauth(&self) -> bool {
        true
    }
}

impl PasswordAuthenticator {
    /// Creates a new `PasswordAuthenticator`. If you do not have a client ID and secret (or do
    /// not know what these are), you need to fetch one using the instructions in the module
    /// documentation.
    pub fn new(client_id: &str,
               client_secret: &str,
               username: &str,
               password: &str)
               -> Arc<Mutex<Box<Authenticator + Send>>> {
        Arc::new(Mutex::new(Box::new(PasswordAuthenticator {
            client_id: client_id.to_owned(),
            client_secret: client_secret.to_owned(),
            username: username.to_owned(),
            password: password.to_owned(),
            access_token: None,
        })))
    }
}
