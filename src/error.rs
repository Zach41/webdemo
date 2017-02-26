use std::error::Error;
use std::{fmt, io};

use modifier::Modifier;
pub use hyper::error::Error as HyperError;

use Response;

#[derive(Debug)]
pub enum WebError {
    Middleware(MiddleError),
    Http(HyperError),
}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WebError::Middleware(ref err) => fmt::Display::fmt(err, f),
            WebError::Http(ref err) => fmt::Display::fmt(err, f),
        }
    }
}

impl Error for WebError {
    fn description(&self) -> &str {
        match *self {
            WebError::Middleware(ref err) => err.description(),
            WebError::Http(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            WebError::Middleware(ref err) => err.cause(),
            WebError::Http(ref err) => err.cause(),
        }
    }
}

impl From<MiddleError> for WebError {
    fn from(err: MiddleError) -> WebError {
        WebError::Middleware(err)
    }
}

impl From<HyperError> for WebError {
    fn from(err: HyperError) -> WebError {
        WebError::Http(err)
    }
}

impl From<io::Error> for WebError {
    fn from(err: io::Error) -> WebError {
        WebError::Http(From::from(err))
    }
}

#[derive(Debug)]
pub struct MiddleError {
    pub error: Box<Error + Send>,
    pub response: Response,
}

impl MiddleError {
    pub fn new<E, M>(err: E, m: M) -> MiddleError
        where E: 'static + Send + Error,
              M: Modifier<Response> {
        MiddleError {
            error: Box::new(err),
            response: Response::with(m),
        }
    }
}

impl fmt::Display for MiddleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&*self.error, f)
    }
}

impl Error for MiddleError {
    fn description(&self) -> &str {
        self.error.description()
    }

    fn cause(&self) -> Option<&Error> {
        self.error.cause()
    }
}

pub type WebResult<T> = Result<T, WebError>;
