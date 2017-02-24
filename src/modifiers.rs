use std::io;
use std::fs::File;
use std::path::{Path, PathBuf};

use Url;
use status::StatusCode;
use headers;
use modifier::{Modifier, Set};
use conduit_mime_types as mime_types;
use hyper::mime::Mime;

use {Request, Response, WriteBody, BodyReader};

lazy_static! {
    static ref MIME_TYPES: mime_types::Types = mime_types::Types::new().unwrap();
}

pub struct Header<H: headers::Header + headers::HeaderFormat>(pub H);

impl<'a, 'b, H> Modifier<Request<'a, 'b>> for Header<H>
    where H: headers::Header + headers::HeaderFormat {
    fn modify(self, request: &mut Request<'a, 'b>) {
        request.headers.set(self.0);
    }
}

impl Modifier<Response> for Mime {
    fn modify(self, res: &mut Response) {
        res.headers.set(headers::ContentType(self));
    }
}

impl Modifier<Response> for StatusCode {
    fn modify(self, res: &mut Response) {
        res.status = Some(self)
    }
}

impl Modifier<Response> for Box<WriteBody> {
    fn modify(self, res: &mut Response) {
        res.body = Some(self);
    }
}

impl<R: io::Read + Send + 'static> Modifier<Response> for BodyReader<R> {
    fn modify(self, res: &mut Response) {
        res.body = Some(Box::new(self));
    }
}

impl Modifier<Response> for Vec<u8> {
    fn modify(self, res: &mut Response) {
        res.headers.set(headers::ContentLength(self.len() as u64));
        res.body = Some(Box::new(self));
    }
}

impl<'a> Modifier<Response> for &'a [u8] {
    fn modify(self, res: &mut Response) {
        self.to_vec().modify(res);
    }
}

impl Modifier<Response> for String {
    fn modify(self, res: &mut Response) {
        self.into_bytes().modify(res);
    }
}

impl<'a> Modifier<Response> for &'a str {
    fn modify(self, res: &mut Response) {
        self.as_bytes().modify(res);
    }
}

impl Modifier<Response> for File {
    fn modify(self, res: &mut Response) {
        if let Ok(metedata) = self.metadata() {
            res.headers.set(headers::ContentLength(metedata.len()));
        }

        res.body = Some(Box::new(self));
    }
}

impl<'a> Modifier<Response> for &'a Path {
    fn modify(self, res: &mut Response) {
        File::open(self)
            .expect(&format!("No such file: {}", self.display()))
            .modify(res);
        let mime_str = MIME_TYPES.mime_for_path(self);
        let _ = mime_str.parse().map(|mime: Mime| res.set_mut(mime));
    }
}

impl Modifier<Response> for PathBuf {
    fn modify(self, res: &mut Response) {
        File::open(&self)
            .expect(&format!("No such file: {}", self.display()))
            .modify(res);

        let mime_str = MIME_TYPES.mime_for_path(&self);
        let _ = mime_str.parse().map(|mime: Mime| res.set_mut(mime));
    }
}


impl<H> Modifier<Response> for Header<H>
    where H: headers::Header + headers::HeaderFormat {
    fn modify(self, res: &mut Response) {
        res.headers.set(self.0);
    }
}

pub struct Redirect(pub Url);

impl Modifier<Response> for Redirect {
    fn modify(self, res: &mut Response) {
        let Redirect(url) = self;
        res.headers.set(headers::Location(url.to_string()));
    }
}

pub struct RedirectRaw(pub String);

impl Modifier<Response> for RedirectRaw {
    fn modify(self, res: &mut Response) {
        let RedirectRaw(path) = self;
        res.headers.set(headers::Location(path));
    }
}
