use std::error::Error;
use std::fmt::{Display, Result as FmtResult, Formatter};
use hyper::status::StatusCode;
use hyper;
use serde_json;

/// Error type that occurs when an API request fails for some reason.
#[derive(Debug)]
pub enum APIError {
    /// Occurs when a listing has run out of results. Only used internally - the `Listing` class
    /// will not raise this when iterating.
    ExhaustedListing,
    /// Occurs when the API has returned a non-success error code. Important status codes include:
    /// - 401 Unauthorized - this usually occurs if your tokens are incorrect or invalid
    /// - 403 Forbidden - you are not allowed to access this, but your request was valid.
    HTTPError(StatusCode),
    /// Occurs if the HTTP response from Reddit was corrupt and Hyper could not parse it.
    HyperError(hyper::Error),
    /// Occurs if JSON deserialization fails. This will always be a bug, so please report it
    /// if it does occur, but the error type is provided so you can fail gracefully.
    JSONError(serde_json::Error),
    /// Occurs if a field that was expected to exist is missing.
    MissingField(&'static str),
}

impl Display for APIError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Error! {}. ({:?})", self.description(), self)
    }
}

impl Error for APIError {
    fn description(&self) -> &str {
        match *self {
            APIError::HTTPError(_) => "The API returned a non-success error code",
            APIError::HyperError(_) => "An error occurred while processing the HTTP response",
            APIError::JSONError(_) => {
                "The JSON sent by Reddit did not match what rawr was expecting"
            }
            _ => "This error should not have occurred. Please file a bug",
        }
    }
}

impl From<hyper::Error> for APIError {
    fn from(err: hyper::Error) -> APIError {
        APIError::HyperError(err)
    }
}

impl From<serde_json::Error> for APIError {
    fn from(err: serde_json::Error) -> APIError {
        APIError::JSONError(err)
    }
}
