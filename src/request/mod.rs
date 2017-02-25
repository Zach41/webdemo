use std::net::SocketAddr;
use std::fmt;

use plugin::{Extensible, Pluggable};
use modifier::Set;
use typemap::TypeMap;

use hyper::server::Request as HyperRequest;
use hyper::{Method, Uri, HttpVersion, Body}; 
use hyper::header::Headers;

pub struct Request {
    req: HyperRequest,
    extensions: TypeMap,
}

impl Request {
    pub fn new(req: HyperRequest) -> Request {
        Request {
            req: req,
            extensions: TypeMap::new(),
        }
    }

    pub fn method(&self) -> &Method {
        self.req.method()
    }

    pub fn headers(&self) -> &Headers {
        self.req.headers()
    }

    pub fn uri(&self) -> &Uri {
        self.req.uri()
    }

    pub fn version(&self) -> &HttpVersion {
        self.req.version()
    }

    pub fn remote_addr(&self) -> Option<&SocketAddr> {
        self.req.remote_addr()
    }

    pub fn path(&self) -> &str {
        self.req.path()
    }

    pub fn query(&self) -> Option<&str> {
        self.req.query()
    }

    pub fn body(self) -> Body {
        self.req.body
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.res, f)
    }
}
// #[derive(Clone, Copy)]
// pub enum Protocol {
//     HTTP,
//     HTTPS,
// }

// impl Protocol {
//     pub fn http() -> Protocol {
//         Protocol::HTTP
//     }

//     pub fn https() -> Protocol {
//         Protocol::HTTPS
//     }

//     pub fn name(&self) -> &str {
//         match *self {
//             Protocol::HTTP => "http",
//             Protocol::HTTPS => "https",
//         }
//     }    
// }

impl Extensible for Request {
    fn extensions(&self) -> &TypeMap {
        &self.extensions
    }

    fn extensions_mut(&mut self) -> &mut TypeMap {
        &mut self.extensions
    }
}

impl Pluggable for Request {}

impl Set for Request {}
