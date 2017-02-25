extern crate webdemo;

use webdemo::prelude::*;

fn content_handle(_: &mut Request) -> WebResult<Response> {
    Ok(Response::with((StatusCode::Ok,
                       webdemo::headers::ContentType::json().0)))
}

fn main() {
    let _ = Web::new(content_handle).http("0.0.0.0:8080").unwrap();
}
