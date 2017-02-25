use std::fmt;

use hyper::server::Response as HyperResponse;
use hyper::{Body, HttpVersion};
use hyper::status::StatusCode;
use hyper::header::Headers;
use typemap::TypeMap;
use plugin::{Pluggable, Extensible};
use modifier::Set;

pub struct Response {
    res: HyperResponse,
    extensions: TypeMap,
}

impl Response {
    pub fn new(res: HyperResponse) -> Response {
        Response {
            res: res,
            extensions: TypeMap::new(),
        }
    }

    pub fn headers(&self) -> &Headers {
        self.res.headers()
    }

    pub fn status(&self) -> &StatusCode {
        self.res.status()
    }

    pub fn version(&self) -> &HttpVersion {
        self.res.version()
    }

    pub fn headers_mut(&mut self) -> &mut Headers {
        self.res.headers_mut()
    }

    pub fn set_status(&mut self, status: StatusCode) {
        self.res.set_status(status)
    }

    pub fn set_body<T: Into<Body>>(&mut self, body: T) {
        self.res.set_body(body)
    }

    // pub fn with_status(self, status: StatusCode) -> Self {
    //     let extensions = self.extensions.clone();
    //     let res = self.res.with_status(status);
    //     Response {
    //         res: res,
    //         inner: extensions,
    //     }
    // }
}

impl Extensible for Response {
    fn extensions(&self) -> &TypeMap {
        &self.extensions
    }

    fn extensions_mut(&mut self) -> &mut TypeMap {
        &mut self.extensions

    }
}

impl Pluggable for Response {}

impl Set for Response {}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.res, f)
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
