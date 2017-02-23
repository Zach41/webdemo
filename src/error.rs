use std::error::Error;
use std::fmt;

use modifier::Modifier;

use Response;

#[derive(Debug)]
pub struct WebError {
    pub error: Box<Error + Send>,
    pub response: Response,
}

impl WebError {
    pub fn new<E, M>(err: E, m: M) -> WebError
        where E: 'static + Send + Error,
              M: Modifier<Response> {
        WebError {
            error: Box::new(err),
            response: Response::with(m),
        }
    }
}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&*self.error, f)
    }
}

impl Error for WebError {
    fn description(&self) -> &str {
        self.error.description()
    }

    fn cause(&self) -> Option<&Error> {
        self.error.cause()
    }
}

pub type WebResult<T> = Result<T, WebError>;
pub use hyper::error::Result as HttpResult;
