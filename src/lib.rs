#![warn(missing_docs)]

//! # rawr - Rust API Wrapper for Reddit
//! This crate provides simple yet powerful access to the Reddit API.
//! ## Quick Start
//! This section will teach you the basics of `rawr`, so you can get started with your project
//! easily.
//! ### Creating a Client
//! To create a client to access the Reddit API, you can use the `RedditClient` class. All you need
//! to do is pass in the user agent when creating the client to have anonymous access. In order
//! to post or reply, you need to log in (which will be explained soon!).
//!
//! As well as passing in a user agent, you need to pass in an *authenticator*, which handles
//! logging in and proving who you are to Reddit. For now, we'll log in anonymously with
//! `AnonymousAuthenticator`. You'll definitely want to take a look at the
//! [auth](./auth/index.html) module documentation to find out how to log in to an account.
//!
//! ```rust,no_run
//! use rawr::client::RedditClient;
//! use rawr::auth::AnonymousAuthenticator;
//! let client = RedditClient::new("my user agent", AnonymousAuthenticator::new());
//! ```
//!
//! It is important that you pick a good user agent. The ideal format is
//! `platform:program:version (by /u/yourname)`, e.g. `linux:rawr:v0.0.1 (by /u/Aurora0001)`.
//!
//! ### Getting a List of Posts
//! First, you'll need to select the subreddit that you want to get a listing from. For our
//! example, we'll use /r/all. To get that subreddit, we can do this:
//!
//! ```rust,no_run
//! # use rawr::client::RedditClient;
//! # use rawr::auth::AnonymousAuthenticator;
//! let client = RedditClient::new("?:rawr:doc-tests", AnonymousAuthenticator::new());
//! let all = client.subreddit("all");
//! ```
//!
//! Once we have the subreddit object, we can get the post list. There are several listing types
//! provided by Reddit:
//! - Hot - the default sort, which shows the 'best' posts at this moment.
//! - New - posts ordered from newest to oldest.
//! - Rising - posts gaining in popularity (usually, this is quite a short listing and only has
//!   10-20 items)
//!
//! There are also 'time-scoped' listings that require a timespan as well. This is essentially
//! the same as the 'links from: all time' dropdown.
//! - Controversial - posts that have received lots of upvotes and downvotes.
//! - Top - posts that have the highest score overall.
//!
//! Here's an example of using the `hot` listing:
//!
//! ```rust,no_run
//! # use rawr::client::RedditClient;
//! # use rawr::auth::AnonymousAuthenticator;
//! use rawr::options::ListingOptions;
//! # let client = RedditClient::new("?:rawr:doc-tests", AnonymousAuthenticator::new());
//! # let all = client.subreddit("all");
//! let listing = all.hot(ListingOptions::default()).expect("Request unsuccessful");
//! ```
//!
//! *Why do I need the `expect` call?* Since it is possible for a request to the API to fail,
//! most of the API wrapper returns `Result<_, APIError>` where `_` is the actual value from the
//! API. This allows you to handle the error gracefully if you want, but for our examples
//! we will always use `expect` to simplify the examples.
//!
//! The `ListingOptions` object is used to configure various options such as the amount of posts
//! to fetch in each request and the anchor. For more information, see the documentation for
//! `ListingOptions`.
//!
//! ### Using the Post Listing
//! The `Listing` objects implement `Iterator`, which means it's really convenient to loop through
//! multiple posts. You can just use a for loop, like this:
//!
//! ```rust,no_run
//! # use rawr::client::RedditClient;
//! # use rawr::auth::AnonymousAuthenticator;
//! # use rawr::options::ListingOptions;
//! # let client = RedditClient::new("?:rawr:doc-tests", AnonymousAuthenticator::new());
//! # let all = client.subreddit("all");
//! let listing = all.hot(ListingOptions::default()).expect("Could not fetch posts");
//! for post in listing {
//!     println!("{}", post.title());
//! }
//! ```
//! ### Getting Replies
//! If you're interested in the replies to a post, you can get them using the `replies()` method.
//! `replies()` returns a `CommentList`, which acts a lot like a `Listing` in that it will
//! automatically fetch more comments as needed. However, unlike post listings, the comments
//! section is threaded, so you can get the `replies()` to a comment as well.
//!
//! Here's how to get the first comment (if available) for every post in the /r/all/hot
//! listing:
//!
//! ```rust,no_run
//! # use rawr::auth::AnonymousAuthenticator;
//! # use rawr::client::RedditClient;
//! # use rawr::options::ListingOptions;
//! use rawr::traits::{Commentable, Content};
//! # let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
//! let all = client.subreddit("all");
//! for post in all.hot(ListingOptions::default()).expect("Request failed") {
//!     if let Some(comment) = post.replies().expect("Could not get replies").next() {
//!         println!("{}", comment.author().name);
//!     }
//! }
//! ```
//!
//! *'What were those traits about?'*, I hear you say. The `Commentable` trait provides
//! functions to work with comments and manipulate them. The `Commentable` trait is implemented
//! in both the `Comment` struct and the `Submission` struct, because you can comment on either.
//! This also explains why you can get the `replies()` for a `Comment` using the same interface.
//!
//! The `Content` trait is a little bit different. Things that implement `Content` were created by
//! a user and exist in a subreddit (so they're either submissions or comments). You can use this
//! trait to delete posts, get the author or get the parent subreddit.
//!
//! ### Replying
//! The `Commentable` trait also allows your client to reply to anything that implements it.
//! You can reply like this:
//!
//! ```rust,no_run
//! # use rawr::auth::AnonymousAuthenticator;
//! # use rawr::client::RedditClient;
//! # use rawr::options::ListingOptions;
//! # use rawr::traits::{Commentable, Content};
//! # let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
//! let all = client.subreddit("all");
//! let mut listing = all.hot(ListingOptions::default()).expect("Request failed");
//! if let Some(top_post) = listing.next() {
//!     top_post.reply("This is cool... I think. I'm a bot.").expect("Posting failed!");
//! }
//! ```
//!
//! Most actions in the API return a `Result` so you can handle any errors that occur. In this
//! example, we just use `expect()`, but real code should handle this more gracefully, otherwise
//! the code may panic. Be prepared to accept HTTP 5xx errors if reddit is having trouble with
//! their servers; this will be returned to you as a `APIError::HTTPError` with the error
//! status in the enum variant.
//! ### Posting a Thread
//! Reddit has two types of submissions: link posts and self posts. Link posts are hyperlinked
//! to their URL, but have no body text. Self posts have a body but don't link anywhere. Both
//! earn karma.
//!
//! To post a link post, you can use the `submit_link()` method. First, you need to create a
//! `LinkPost` object that can configure your post, like this:
//!
//! ```
//! use rawr::options::LinkPost;
//! let post = LinkPost::new("title", "https://example.com");
//! ```
//!
//! If that link has been posted previously in the subreddit, you need to mark it as a repost, like
//! this:
//!
//! ```
//! # use rawr::options::LinkPost;
//! let post = LinkPost::new("title", "https://example.com").resubmit();
//! ```
//!
//! You can then submit it, like this:
//!
//! ```rust,no_run
//! # use rawr::auth::AnonymousAuthenticator;
//! # use rawr::client::RedditClient;
//! # use rawr::options::ListingOptions;
//! # use rawr::traits::{Commentable, Content};
//! use rawr::options::LinkPost;
//! # let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
//! let programming = client.subreddit("programming");
//! let post = LinkPost::new("I love Rust!", "https://rust-lang.org");
//! programming.submit_link(post).expect("Could not submit link!");
//! ```
//!
//! Submitting a self post is largely the same, but the `resubmit` function is not available
//! on the `SelfPost` struct. Example:
//!
//! ```rust,no_run
//! # use rawr::auth::AnonymousAuthenticator;
//! # use rawr::client::RedditClient;
//! # use rawr::options::ListingOptions;
//! # use rawr::traits::{Commentable, Content};
//! use rawr::options::SelfPost;
//! # let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
//! let programming = client.subreddit("programming");
//! let post = SelfPost::new("I love Rust!", "It's great! **Wow**!");
//! programming.submit_text(post).expect("Could not submit link!");
//! ```
//!
//! # More Advanced Features
//! `rawr` also supports:
//! - Getting a user by name (see `RedditClient.user("name")`)
//! - Setting user flairs (see the `user` module)
//! - Setting post flairs (see `Submission.flair`)
//! - Getting 'about' information (see `Subreddit.about()`)
//!
//! # Not Yet Implemented
//! - Multireddits
//! - Live Threads

extern crate serde;
extern crate serde_json;
extern crate hyper;

pub mod auth;
pub mod client;
/// Deserialized API responses.
pub mod responses;
/// Traits for common API functionality.
pub mod traits;
/// Error types used to signal problems with requests.
pub mod errors;
/// Reddit structures such as submissions, comments and subreddits.
pub mod structures;
/// Configuration options for API requests.
pub mod options;
/// Basic `rawr` structures to import with `use rawr::prelude::*`;
pub mod prelude;

#[cfg(test)]
mod tests {
    use client::RedditClient;
    use structures::submission::Submission;
    use options::ListingOptions;
    use auth::AnonymousAuthenticator;
    #[test]
    fn hot_length() {
        let client = RedditClient::new("rawr", AnonymousAuthenticator::new());
        let r_all = client.subreddit("all");
        let hot = r_all.hot(ListingOptions::default()).expect("Request failed!");
        let hot_list = hot.take(26).collect::<Vec<Submission>>();
        assert_eq!(hot_list.len() as usize, 26);
    }
}
