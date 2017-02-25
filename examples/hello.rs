extern crate webdemo;

use webdemo::prelude::*;

fn main() {
    let serv = Web::new(|req: &mut Request| {
        println!("{:?}", req);
        Ok(Response::with((StatusCode::Ok, "Hello, there")))
    }).http("0.0.0.0:8080");
}
