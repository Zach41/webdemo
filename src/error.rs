use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct WebError;

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("WebError")
    }
}
impl StdError for WebError {
    fn description(&self) -> &str {
        "WebError"
    }
}

pub type WebResult<T> = Result<T, WebError>;
