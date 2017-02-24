extern crate webdemo;

use webdemo::{Request, Response, WebResult, status, Web};

fn content_handle(_: &mut Request) -> WebResult<Response> {
    Ok(Response::with((status::StatusCode::Ok,
                       webdemo::headers::ContentType::json().0)))
}

fn main() {
    let _ = Web::new(content_handle).http("0.0.0.0:8080").unwrap();
}
