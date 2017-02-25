extern crate webdemo;
extern crate time;
extern crate env_logger;

use webdemo::prelude::*;
use time::precise_time_ns;

struct ResponseTime;

impl types::Key for ResponseTime { type Value = u64; }

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> WebResult<()> {
        req.extensions.insert::<ResponseTime>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, res: Response) -> WebResult<Response> {
        let delta = precise_time_ns() - *req.extensions.get::<ResponseTime>().unwrap();
        println!("Request took: {} ms", (delta as f64) / 1000000.0);
        Ok(res)
    }
}

fn main() {
    let _ = env_logger::init();
    let mut chain = Chain::new(|req: &mut Request| {
        println!("{:?}", req);
        Ok(Response::with((StatusCode::Ok, "Hello, World!")))
    });
    chain.before(ResponseTime);
    chain.after(ResponseTime);
    Web::new(chain).http("0.0.0.0:8080").unwrap();
}
