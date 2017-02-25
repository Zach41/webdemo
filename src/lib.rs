extern crate error as err;
extern crate url as url_ext;
extern crate hyper;
extern crate typemap;
extern crate plugin;
extern crate modifier;
extern crate conduit_mime_types;
#[macro_use]
extern crate lazy_static;
extern crate num_cpus;
#[macro_use]
extern crate log;

mod request;
mod response;
mod error;
mod middleware;
mod modifiers;

pub use request::{Request, Url, Body, Protocol};
pub use response::{Response, WriteBody, BodyReader, ResponseBody};
pub use error::{WebError, WebResult, HttpResult};
pub use middleware::{BeforeMiddleware, AfterMiddleware, AroundMiddleware, Handler, Chain};
pub use method::Method;
pub use modifiers::{Header, Redirect, RedirectRaw};

pub use hyper::header as headers;
pub use hyper::status as status;

use std::time::Duration;
use std::net::{SocketAddr, ToSocketAddrs};

use hyper::server::{Handler as HyperHandler, Request as HyperRequest, Response as HyperResponse};
use hyper::server::{Server, Listening};
use hyper::net::{Fresh, SslServer, NetworkListener};
use hyper::net::{HttpListener, HttpsListener};

mod method {
    pub use hyper::method::Method;
}

pub mod types {
    pub use typemap::*;
}

pub mod prelude {
    pub use {Request, Response, Url, Protocol, WebResult, WebError, HttpResult};
    pub use {BeforeMiddleware, AfterMiddleware, Chain, AroundMiddleware, Handler};
    pub use status::*;
    pub use Method;
    pub use {Web, Timeout};
    pub use types;
}

pub struct Web<H> {
    handler: H,
    timeouts: Timeout,
    threads: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timeout {
    pub keep_alive: Option<Duration>,
    pub read_timeout: Option<Duration>,
    pub write_timeout: Option<Duration>,
}

impl Default for Timeout {
    fn default() -> Timeout {
        Timeout {
            keep_alive: Some(Duration::from_secs(5)),
            read_timeout: Some(Duration::from_secs(30)),
            write_timeout: Some(Duration::from_secs(1)),
        }
    }
}

struct RawHandler<H> {
    handler: H,
    protocol: Protocol,
    addr: SocketAddr,
}

impl<H: Handler> HyperHandler for RawHandler<H> {
    fn handle<'a, 'k>(&'a self, req: HyperRequest<'a, 'k>, mut res:HyperResponse<'a, Fresh>) {
        *res.status_mut() = status::StatusCode::InternalServerError;

        match Request::from_http(req, self.addr, &self.protocol) {
            Ok(mut request) => {
                self.handler.handle(&mut request).unwrap_or_else(|e| {
                    error!("Error when handling request: {:?}, error: {:?}", request, e);
                    e.response
                }).write_back(res);                
            },
            Err(e) => {
                error!("Error creating request: {}", e);
                bad_request(res);
            }
        }
    }
}

fn bad_request(mut http_res: HyperResponse<Fresh>) {
    *http_res.status_mut() = status::StatusCode::BadRequest;

    if let Ok(res) = http_res.start() {
        let _ = res.end();
    }
}

impl<H: Handler> Web<H> {
    pub fn new(handler: H) -> Web<H> {
        Web {
            handler: handler,
            timeouts: Timeout::default(),
            threads: ::num_cpus::get(),
        }
    }

    pub fn http<A: ToSocketAddrs>(self, addr: A) -> HttpResult<Listening> {
        HttpListener::new(addr).and_then(|l| self.listen(l, Protocol::HTTP))
    }

    // pub fn https<A, S>(self, addr: A, ssl: S) -> HttpResult<Listening>
    //     where A: ToSocketAddrs,
    //           S: 'static + SslServer + Send + Clone {
    //     HttpsListener::new(addr, ssl).and_then(|l| self.listen(l, Protocol::HTTPS))
    // }

    pub fn listen<L>(self, mut listener: L, protocol: Protocol) -> HttpResult<Listening>
        where L: 'static + NetworkListener + Send {
        let handler = RawHandler {
            handler: self.handler,
            protocol: protocol,
            addr: try!(listener.local_addr()),
        };

        let mut server = Server::new(listener);
        server.keep_alive(self.timeouts.keep_alive);
        server.set_read_timeout(self.timeouts.read_timeout);
        server.set_write_timeout(self.timeouts.write_timeout);
        server.handle_threads(handler, self.threads)
    }
}
