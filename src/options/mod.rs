use std::fmt::{Display, Formatter, Result as FmtResult};

/// Configures a paginated listing.
pub struct ListingOptions {
    /// The maximum amount of posts to fetch in one request.
    pub batch: u8,
    /// See `ListingAnchor` for explanation of this property.
    pub anchor: ListingAnchor,
}

impl Display for ListingOptions {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "limit={}", self.batch)
    }
}

impl ListingOptions {
    /// Provides the default options (25 posts per page, starts at first post in listing). If
    /// you are unsure, this will act like the default display options on Reddit.
    pub fn default() -> ListingOptions {
        ListingOptions {
            batch: 25,
            anchor: ListingAnchor::None,
        }
    }
}

/// Used to 'anchor' the pagination so you can get all posts before/after a post.
pub enum ListingAnchor {
    /// Gets all items after the specified one, e.g. gets posts older than the specified post in
    /// the new queue.
    After(String),
    /// Gets all items before the specified one, e.g. gets posts higher than the specified one in
    /// the top queue.
    Before(String),
    /// Use no anchor.
    None,
}

impl Display for ListingAnchor {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            ListingAnchor::Before(ref pos) => write!(f, "before={}", pos),
            ListingAnchor::After(ref pos) => write!(f, "after={}", pos),
            ListingAnchor::None => Ok(()),
        }
    }
}

/// Used for filtering by time in the top and controversial queues.
#[allow(missing_docs)]
pub enum TimeFilter {
    Hour,
    Day,
    Week,
    Month,
    Year,
    AllTime,
}

impl Display for TimeFilter {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let s = match *self {
            TimeFilter::Hour => "hour",
            TimeFilter::Day => "day",
            TimeFilter::Week => "week",
            TimeFilter::Month => "month",
            TimeFilter::Year => "year",
            TimeFilter::AllTime => "all",
        };
        write!(f, "&t={}", s)
    }
}

/// Options used when creating a link post. See `structures::subreddit` for examples of usage.
pub struct LinkPost {
    /// The title of the link post to create
    pub title: String,
    /// The URL of the link to post. If this has been posted before, you **must** mark this as a
    /// resubmission or the API will raise an error. See `LinkPost::resubmit()` for an example.
    pub link: String,
    /// True if resubmitting this link is intended.
    pub resubmit: bool,
}

impl LinkPost {
    /// Creates a new `LinkPost` structure that contains the options for a link post.
    /// The post is not actually sent at this point.
    pub fn new(title: &str, link: &str) -> LinkPost {
        LinkPost {
            title: title.to_owned(),
            link: link.to_owned(),
            resubmit: false,
        }
    }

    /// Marks this post as a resubmission, so the API will accept it even if it has been submitted
    /// in this subreddit before.
    /// # Examples
    /// ```
    /// use rawr::options::LinkPost;
    /// let post = LinkPost::new("Look at this!", "http://example.com/foo").resubmit();
    /// ```
    pub fn resubmit(mut self) -> LinkPost {
        self.resubmit = true;
        self
    }
}

/// Options used when creating a self post. See `structures::subreddit` for examples of usage.
pub struct SelfPost {
    /// The title of the link post to create
    pub title: String,
    /// The markdown post body.
    pub text: String,
}

impl SelfPost {
    /// Creates a new `SelfPost` object. The post will not be submitted until you use
    /// `Subreddit.submit_text()`.
    pub fn new(title: &str, text: &str) -> SelfPost {
        SelfPost {
            title: title.to_owned(),
            text: text.to_owned(),
        }
    }
}
