use std::error::Error;
use std::fmt;
use std::io::Error as IOError;

use modifier::Modifier;

pub use hyper::error::Error as HyperError;

use Response;

#[derive(Debug)]
pub enum WebError {
    Middleware(MiddleError),
    HTTP(HyperError),
}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WebError::Middleware(ref err) => fmt::Display::fmt(err, f),
            WebError::HTTP(ref err) => fmt::Display::fmt(err, f),
        }
    }
}

impl Error for WebError {
    fn description(&self) -> &str {
        match *self {
            WebError::Middleware(ref err) => err.description(),
            WebError::HTTP(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            WebError::Middleware(ref err) => err.cause(),
            WebError::HTTP(ref err) => err.cause(),
        }
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

impl From<HyperError> for WebError {
    fn from(err: HyperError) -> WebError {
        WebError::HTTP(err)
    }
}

impl From<MiddleError> for WebError {
    fn from(err: MiddleError) -> WebError {
        WebError::Middleware(err)
    }
}

impl From<IOError> for WebError {
    fn from(err: IOError) -> WebError {
        WebError::HTTP(From::from(err))
    }
}

pub type WebResult<T> = Result<T, WebError>;
// pub use hyper::error::Result as HttpResult;
