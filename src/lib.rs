extern crate error as err;
extern crate url as url_ext;
extern crate hyper;
extern crate typemap;
extern crate plugin;
extern crate modifier;
extern crate conduit_mime_types;
#[macro_use]
extern crate lazy_static;

mod request;
mod response;
mod error;
mod middleware;
mod modifiers;

pub use request::{Request, Url};
pub use response::{Response, WriteBody, BodyReader};
pub use error::{WebError, WebResult};
pub use middleware::{BeforeMiddleware, AfterMiddleware, AroundMiddleware, Handler, Chain};
pub use method::Method;

pub use hyper::header as headers;
pub use hyper::status as status;

mod method {
    pub use hyper::method::Method;
}

