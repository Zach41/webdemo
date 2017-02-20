extern crate error as err;
extern crate url as url_ext;
extern crate hyper;
extern crate typemap;

mod request;
mod response;
mod error;
mod middleware;

pub use request::{Request, Url};
pub use response::Response;
pub use error::{WebError, WebResult};
pub use middleware::{BeforeMiddleware, AfterMiddleware, AroundMiddleware, Handler, Chain};
pub use method::Method;
pub use hyper::header as headers;

mod method {
    pub use hyper::method::Method;
}

