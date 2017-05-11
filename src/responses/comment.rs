use serde_json::Value;
use responses::{BasicThing, ThingList};
use responses::listing::ListingData;
use util::deserialize_timestamp;

/// The 'listing' format for comments.
pub type CommentListing = BasicThing<ListingData<Value>>;

pub type MoreComments = JSONWrapper<ThingList>;

pub type NewComment = JSONWrapper<CommentThings>;


/// A deserializable structure representing a comment. This is created when the client returns
/// JSON representing a comment and this is wrapped in a `models::comment::Comment` for
/// ease-of-use.
#[derive(Deserialize, Debug)]
pub struct Comment {
    /// The Reddit ID for the subreddit where this was posted, **including the leading `t5_`**.
    pub subreddit_id: String,
    /// Contains the name of the moderator who banned this, if the logged-in user is a moderator
    /// of this subreddit and this is banned.
    pub banned_by: Option<String>,
    /// The reason for the post removal, if you are a moderator **and** this post has been
    /// removed.
    pub removal_reason: Option<String>,
    /// The ID (including kind) of the parent post (i.e. the link post or self post).
    pub link_id: String,
    /// This is `Some(true)` if the logged-in user has upvoted this submission, `Some(false)` if
    /// the user has downvoted this submission or `None` if the user has not voted.
    pub likes: Option<bool>,
    /// A listing of replies to this comment. Do not access this directly - use the functions
    /// of `Comment` instead.
    pub replies: Value,
    // TODO: skipped user_reports
    /// True if the logged-in user has saved this submission.
    pub saved: bool,
    /// The ID of the post in base-36 form, as used in Reddit's links.
    pub id: String,
    /// The amount of times that a user has been gilded (gifted Reddit Gold).
    pub gilded: u64,
    /// This is `true` if Reddit has archived the submission (usually done after 6 months).
    /// Archived submissions cannot be voted or commented upon.
    pub archived: bool,
    // TODO: skipped report_reasons
    /// The name of the author of the submission (not including the leading `/u/`)
    pub author: String,
    /// The overall points score of this post, as shown on the upvote counter. This is the
    /// same as upvotes - downvotes (however, this figure may be fuzzed by Reddit, and may not
    /// be exact)
    pub score: i64,
    /// This contains the name of the user who approved this submission. This is `None` unless
    /// you are a mod of the subreddit **and** a user has approved this post.
    pub approved_by: Option<String>,
    // TODO: skipped controversiality
    /// The main post text.
    pub body: String,
    /// This is `false` if the submission is not edited and is the edit timestamp if it is edited.
    /// Access through the functions of `Submission` instead.
    pub edited: Value,
    /// The CSS class set for the author's flair (if available). If there is no flair, this is
    /// `None`.
    pub author_flair_css_class: Option<String>,
    /// The number of downvotes (fuzzed; see `score` for further explanation)
    pub downs: i64,
    /// The number of upvotes (fuzzed; see `score` for further explanation)
    pub ups: i64,
    /// Contains the HTML of the post body.
    pub body_html: String,
    /// The subreddit that this submission was posted in (not including `/r/`)
    pub subreddit: String,
    /// The full 'Thing ID', consisting of a 'kind' and a base-36 identifier. The valid kinds are:
    /// - t1_ - Comment
    /// - t2_ - Account
    /// - t3_ - Link
    /// - t4_ - Message
    /// - t5_ - Subreddit
    /// - t6_ - Award
    /// - t8_ - PromoCampaign
    pub name: String,
    /// `true` if the score should not be displayed.
    pub score_hidden: bool,
    /// This is `true` if this submission is stickied (an 'annoucement' thread)
    pub stickied: bool,
    /// A timestamp of the time when the post was created, in the logged-in user's **local**
    /// time.
    #[serde(deserialize_with="deserialize_timestamp")]
    pub created: i64,
    /// The text of the author's flair, if present. Can be an empty string if the flair is present
    /// but contains no text.
    pub author_flair_text: Option<String>,
    /// A timestamp of the time when the post was created, in **UTC**.
    #[serde(deserialize_with="deserialize_timestamp")]
    pub created_utc: i64,
    /// Indicates whether the user has used a special flag for themselves, e.g. [M] or [A].
    /// Possible values:
    /// - None - Normal user
    /// - Some("moderator") - [M]
    /// - Some("admin") - [A]
    /// - Some("special") - other special 'distinguishes' e.g. [Δ]
    pub distinguished: Option<String>,
    pub num_reports: Option<u64>, // TODO: skipped mod_reports
    pub parent_id: String,
}

#[derive(Deserialize, Debug)]
pub struct More {
    pub count: u64,
    pub parent_id: String,
    pub children: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct JSONWrapper<T> {
    pub json: JSONInner<T>,
}

#[derive(Deserialize, Debug)]
pub struct JSONInner<T> {
    pub errors: Vec<Value>,
    pub data: T,
}

#[derive(Deserialize, Debug)]
pub struct CommentThings {
    pub things: Vec<BasicThing<Comment>>,
}
