use std::io::{self, Write, Read};
use std::fs::File;
use std::fmt;

use typemap::TypeMap;
use status::StatusCode;
use headers::{self, Headers};
use modifier::{Modifier, Set};
use plugin::{Extensible, Pluggable};
use hyper::server::response::Response as HyperResponse;
use hyper::net::Fresh;

pub struct Response {
    pub status: Option<StatusCode>,
    pub headers: Headers,
    pub extensions: TypeMap,
    pub body: Option<Box<WriteBody>>,
}

impl Default for Response {
    fn default() -> Self {
        Response::new()
    }
}

impl Response {
    pub fn new() -> Response {
        Response {
            status: None,
            headers: Headers::new(),
            extensions: TypeMap::new(),
            body: None,
        }
    }

    pub fn with<M: Modifier<Response>>(m: M) -> Response {
        let res = Response::new();
        res.set(m)
    }

    pub fn write_back(self, mut http_res: HyperResponse<Fresh>) {
        *http_res.headers_mut() = self.headers;
        *http_res.status_mut() = self.status.unwrap_or(StatusCode::NotFound);

        let out = match self.body {
            Some(b) => write_with_body(http_res, b),
            None => {
                http_res.headers_mut().set(headers::ContentLength(0));
                http_res.start().and_then(|res| res.end())
            }
        };

        if let Err(e) = out {
            error!("Error writing response: {}", e);
        }
    }
}

fn write_with_body(mut http_res: HyperResponse<Fresh>, mut body: Box<WriteBody>) -> io::Result<()> {
    let content_type = http_res.headers().get::<headers::ContentType>()
        .map_or_else(headers::ContentType::plaintext,
                     |cx| cx.clone());
    http_res.headers_mut().set(content_type);

    let mut raw_res = try!(http_res.start());
    try!(body.write_body(&mut ResponseBody::new(&mut raw_res)));
    raw_res.end()
}

pub trait WriteBody: Send {
    fn write_body(&mut self, res: &mut ResponseBody) -> io::Result<()>;
}

pub struct ResponseBody<'a>(Box<Write + 'a>);

impl<'a> ResponseBody<'a> {
    pub fn new<W: Write + 'a>(w: W) -> ResponseBody<'a> {
        ResponseBody ( Box::new(w) )
    }
}

pub struct BodyReader<R: Send>(pub R);

impl<'a> Write for ResponseBody<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl WriteBody for Vec<u8> {
    fn write_body(&mut self, res: &mut ResponseBody) -> io::Result<()> {
        res.write_all(self.as_ref())
    }
}

impl<'a> WriteBody for &'a [u8] {
    fn write_body(&mut self, res: &mut ResponseBody) -> io::Result<()> {
        res.write_all(self)
    }
}

impl<'a> WriteBody for &'a str {
    fn write_body(&mut self, res: &mut ResponseBody) -> io::Result<()> {
        res.write_all(self.as_bytes())
    }
}

impl WriteBody for String {
    fn write_body(&mut self, res: &mut ResponseBody) -> io::Result<()> {
        res.write_all(self.as_bytes())
    }
}

impl WriteBody for File {
    fn write_body(&mut self, res: &mut ResponseBody) -> io::Result<()> {
        io::copy(self, res).map(|_| ())
    }
}

impl<R: Read + Send> WriteBody for BodyReader<R>
    where R: Send + Read {
    fn write_body(&mut self, res: &mut ResponseBody) -> io::Result<()> {
        io::copy(&mut self.0, res).map(|_| ())
    }
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
        writeln!(f, "HTTP/1.1 {}\n{}",
                 self.status.unwrap_or(StatusCode::NotFound),
                 self.headers)
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
