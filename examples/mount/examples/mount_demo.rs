extern crate mount;
extern crate webdemo;
extern crate env_logger;

use webdemo::prelude::*;
use mount::Mount;

fn hello(req: &mut Request) -> WebResult<Response> {
    println!("Path: {:?}", req.url.path());
    Ok(Response::with((StatusCode::Ok, "Hello, World!")))
}

fn main() {
    let _ = env_logger::init().unwrap();
    
    let mut first = Mount::new();
    let mut second = Mount::new();

    second.mount("/level2", hello);
    first.mount("/level1", second);

    Web::new(first).http("0.0.0.0:8080").unwrap();
}
