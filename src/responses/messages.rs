use responses::listing::ListingData;
use responses::BasicThing;
use util::deserialize_timestamp;

use serde_json::Value;

pub type MessageListing = BasicThing<ListingData<Message>>;

#[derive(Deserialize, Debug)]
pub struct Message {
    pub author: Option<String>,
    pub body: String,
    pub body_html: String,
    pub context: String,
    pub first_message_name: Option<String>,
    pub likes: Option<bool>,
    pub name: String,
    pub link_title: Option<String>,
    pub parent_id: Option<String>,
    pub replies: Value,
    pub subject: String,
    pub subreddit: Option<String>,
    pub was_comment: bool,
    #[serde(deserialize_with="deserialize_timestamp")]
    pub created: i64,
    #[serde(deserialize_with="deserialize_timestamp")]
    pub created_utc: i64,
}
