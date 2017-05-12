#![allow(missing_docs)]

pub mod listing;
pub mod auth;
pub mod comment;
pub mod user;
pub mod messages;

use serde_json::Value;

/// A base structure that can represent both 'Thing' objects and 'Listing' objects, which both
/// return a `kind` and `data`.
#[derive(Deserialize, Debug)]
pub struct BasicThing<T> {
    /// An identifier that specifies the type of object that this is.
    /// The valid kinds are:
    /// - t1_ - Comment
    /// - t2_ - Account
    /// - t3_ - Link
    /// - t4_ - Message
    /// - t5_ - Subreddit
    /// - t6_ - Award
    /// - t8_ - PromoCampaign
    pub kind: String,
    /// The data contained by this struct. This will vary depending on the type parameter
    /// because each endpoint returns different contents.
    pub data: T,
}

#[derive(Deserialize, Debug)]
pub struct ThingList {
    pub things: Vec<BasicThing<Value>>,
}

#[derive(Deserialize, Debug)]
pub struct FlairSelectorResponse {
    pub current: CurrentFlairResponse,
    pub choices: Vec<FlairChoice>,
}

#[derive(Deserialize, Debug)]
pub struct FlairChoice {
    pub flair_css_class: String,
    pub flair_template_id: String,
    pub flair_text: String,
    pub flair_position: String,
    pub flair_text_editable: bool,
}

#[derive(Deserialize, Debug)]
pub struct CurrentFlairResponse {
    pub flair_css_class: Option<String>,
    pub flair_template_id: Option<String>,
    pub flair_text: Option<String>,
    pub flair_position: Option<String>,
}
