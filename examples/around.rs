extern crate webdemo;
extern crate time;

use webdemo::prelude::*;
use time::precise_time_ns;

struct Logger;

struct LoggerHandler {
    inner: Box<Handler>
}

impl LoggerHandler {
    fn new(handler: Box<Handler>) -> LoggerHandler {
        LoggerHandler {
            inner: handler,
        }
    }
}

impl Handler for LoggerHandler {
    fn handle(&self, req: &mut Request) -> WebResult<Response> {
        println!("Logging Time: {}", precise_time_ns());
        self.inner.handle(req)
    } 
}

impl AroundMiddleware for Logger {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(
            LoggerHandler {
                inner: handler,
            }
        )
    }
}

fn main() {
    Web::new(
        Logger.around(Box::new(|_: &mut Request| {
            Ok(Response::with((StatusCode::Ok, "Hello, World!")))
        }))).http("0.0.0.0:8080").unwrap();
}
