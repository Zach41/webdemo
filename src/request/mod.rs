mod url;

use std::net::SocketAddr;
use std::io::{self, Read};
use std::fmt;

use hyper::version::HttpVersion;
use hyper::http::h1::HttpReader;
use hyper::net::NetworkStream;
use hyper::buffer::BufReader;
use hyper::uri::RequestUri;
use hyper::server::request::Request as HyperRequest;
use typemap::TypeMap;
use plugin::Pluggable;
use plugin::Extensible;
use modifier::Set;

use Method;
use headers::{Headers, Host};
pub use self::url::Url;

pub struct Request<'a, 'b: 'a> {
    pub url: Url,
    pub method: Method,    
    pub remote_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub headers: Headers,
    pub body: Body<'a, 'b>,
    pub extensions: TypeMap,
    pub version: HttpVersion,
}

pub struct Body<'a, 'b: 'a> {
    inner: HttpReader<&'a mut BufReader<&'b mut NetworkStream>>,
}

impl<'a, 'b> Body<'a, 'b> {
    pub fn new(reader: HttpReader<&'a mut BufReader<&'b mut NetworkStream>>) -> Body<'a, 'b> {
        Body {
            inner: reader,
        }
    }
}

impl<'a, 'b> Read for Body<'a, 'b> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<'a, 'b> Request<'a, 'b> {
    pub fn from_http(req: HyperRequest<'a, 'b>, local_addr: SocketAddr, protocol: &Protocol) -> Result<Request<'a, 'b>, String> {
        let (remote_addr, method, headers, uri, version, reader) = req.deconstruct();

        let url = match uri {
            RequestUri::AbsoluteUri(ref url) => {
                match Url::from_generic_url(url.clone()) {
                    Ok(url) => url,
                    Err(e) => return Err(e),
                }
            },
            RequestUri::AbsolutePath(ref path) => {
                let url_string = match (version, headers.get::<Host>()) {
                    (_, Some(host)) => {
                        if let Some(port) = host.port {
                            format!("{}://{}:{}{}", protocol.name(), host.hostname, port, path)
                        } else {
                            format!("{}://{}{}", protocol.name(), host.hostname, path)
                        }
                    },
                    (version, None) if version < HttpVersion::Http11 => {
                        // attempt to use local address
                        match local_addr {
                            SocketAddr::V4(addr4) => format!("{}://{}:{}{}",
                                                             protocol.name(),
                                                             addr4.ip(),
                                                             addr4.port(),
                                                             path),
                            SocketAddr::V6(addr6) => format!("{}://[{}]:{}{}",
                                                             protocol.name(),
                                                             addr6.ip(),
                                                             addr6.port(),
                                                             path),
                        }
                    },
                    _ => {
                        return Err("No host sepecified in request".into())
                    }
                };

                match Url::parse(&url_string) {
                    Ok(url) => url,
                    Err(e) => return Err(e),
                }
            }
            _ => return Err("Unsupported request URI".into()),
        };

        Ok(Request {
            url: url,
            remote_addr: remote_addr,
            local_addr: local_addr,
            headers: headers,
            body: Body::new(reader),
            extensions: TypeMap::new(),
            version: version,
            method: method,
        })
    }
}

impl<'a, 'b> fmt::Debug for Request<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "Request {{"));
        try!(writeln!(f, "\t\tmethod: {:?}", self.method));
        try!(writeln!(f, "\t\tremote_addr: {:?}", self.remote_addr));
        try!(writeln!(f, "\t\tlocal_addr: {:?}", self.local_addr));
        try!(writeln!(f, "\t\theaders: {:?}", self.headers));

        try!(writeln!(f, "}}"));
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub enum Protocol {
    HTTP,
    HTTPS,
}

impl Protocol {
    pub fn http() -> Protocol {
        Protocol::HTTP
    }

    pub fn https() -> Protocol {
        Protocol::HTTPS
    }

    pub fn name(&self) -> &str {
        match *self {
            Protocol::HTTP => "http",
            Protocol::HTTPS => "https",
        }
    }    
}

impl<'a, 'b> Extensible for Request<'a, 'b> {
    fn extensions(&self) -> &TypeMap {
        &self.extensions
    }

    fn extensions_mut(&mut self) -> &mut TypeMap {
        &mut self.extensions
    }
}

impl<'a, 'b> Pluggable for Request<'a, 'b> {}

impl<'a, 'b> Set for Request<'a, 'b> {}
