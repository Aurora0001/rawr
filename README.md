# rawr
[![Gitter](https://badges.gitter.im/Aurora0001/rawr.svg)](https://gitter.im/Aurora0001/rawr?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)
[![Documentation](https://img.shields.io/badge/documentation-available-green.svg)](https://aurora0001.github.io/rawr/doc/rawr/index.html)

Welcome to rawr, the **R**ust **A**PI **W**rapper for **R**eddit. This library provides simple, pain-free access to the Reddit API 
in Rust. If you want to create a Reddit bot, scrape data from the API or create a web application powered by Reddit's API, rawr can help
to do this simply and effectively.

## Features

- Support for password-based authentication (with OAuth support, so your app is future-proof!) and anonymous authentication.
- Iterable post and comment listings that automatically fetch data from the API as necessary.
- 'Streams' to fetch the latest threads/comments/messages that update automatically by polling the API - ideal for bots that 
react to new posts.

## Examples
```rust
    extern crate rawr;
    use rawr::prelude::*;
    
    fn main() {
        // Creates a new client to access the reddit API. You need to set a user agent so Reddit knows
        // who is using this client.
        let client = RedditClient::new("your user agent here", AnonymousAuthenticator::new());
        // Access the subreddit /r/rust.
        let subreddit = client.subreddit("rust");
        // Gets the hot listing of /r/rust. If the API request fails, we will panic with `expect`.
        let hot_listing = subreddit.hot(ListingOptions::default()).expect("Could not fetch post listing!");
        // Iterates through the top 50 posts of /r/rust. If you do not `take(n)`, this iterator will
        // continue forever!
        for post in hot_listing.take(50) {
            println!("{}", post.title());
        }
    }
```
