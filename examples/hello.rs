extern crate webdemo;

use webdemo::{Handler, Request, Response, status, WebResult, Web};

fn main() {
    let serv = Web::new(|req: &mut Request| {
        println!("{:?}", req);
        Ok(Response::with((status::StatusCode::Ok, "Hello, there")))
    }).http("0.0.0.0:8080");
}
