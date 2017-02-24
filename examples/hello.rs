extern crate webdemo;

use webdemo::{Handler, Request, Response, status, WebResult, Web};

fn main() {
    let serv = Web::new(|_: &mut Request| {
        Ok(Response::with((status::StatusCode::Ok, "Hello, there")))
    }).http("0.0.0.0:8080");
}
