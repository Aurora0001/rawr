//! Responses from user-related API endpoints such as `about` data.

use responses::BasicThing;
/// API response for /user/username/about
pub type UserAbout = BasicThing<UserAboutData>;

#[derive(Deserialize, Debug)]
pub struct UserAboutData {
    pub name: String,
    pub is_friend: bool,
    pub hide_from_robots: bool,
    pub id: String,
    pub created: i64,
    pub created_utc: i64,
    pub link_karma: i64,
    pub comment_karma: i64,
    pub is_gold: bool,
    pub is_mod: bool,
    pub has_verified_email: bool
}
