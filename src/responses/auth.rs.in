//! Authentication-related API response structures

#![allow(missing_docs, doc_markdown, unknown_lints)]

/// A response providing an access token from /api/v1/access_token which can be used for the
/// OAuth-based authenticators
#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub scope: String,
    pub token_type: String
}
