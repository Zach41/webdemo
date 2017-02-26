extern crate webdemo;
extern crate sequence_trie;
#[macro_use]
extern crate log;

use std::path::{Path, Component};
use std::error::Error;
use std::fmt;

use webdemo::types;
use webdemo::prelude::*;
use sequence_trie::SequenceTrie;

#[derive(Clone, Copy, Debug)]
pub struct OriginalUrl;

impl types::Key for OriginalUrl { type Value = Url; }

pub struct Mount {
    inner: SequenceTrie<String, Match>,
}

struct Match {
    handler: Box<Handler>,
    length: usize,
}

impl fmt::Debug for Match {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Match Length: {}", self.length)
    }
}

#[derive(Debug)]
struct NoMatch;

impl Error for NoMatch {
    fn description(&self) -> &str {
        "No Match"
    }
}

impl fmt::Display for NoMatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl Mount {
    pub fn new() -> Self {
        Mount {
            inner: SequenceTrie::new(),
        }
    }

    pub fn mount<H: Handler>(&mut self, route: &str, handler: H) -> &mut Mount {
        let key: Vec<&str> = Path::new(route).components().flat_map(|c| {
            match c {
                Component::RootDir => None,
                _ => Some(c.as_os_str().to_str().unwrap()),
            }
        }).collect();

        let length = key.len();
        self.inner.insert(key, Match {
            handler: Box::new(handler) as Box<Handler>,
            length: length,
        });
        self
    }
}

impl Handler for Mount {
    fn handle(&self, req: &mut Request) -> WebResult<Response> {
        let matched = {
            let path = req.url.path();
            debug!("Path: {:?}", path);

            let key = match path.last() {
                Some(s) if s.is_empty() => &path[..path.len() - 1],
                _ => &path,
            };
            debug!("Key: {:?}", key);

            let key: Vec<_> = key.into_iter().map(|k| String::from(*k) ).collect();

            match self.inner.get_ancestor(&key) {
                Some(m) => m,
                None => return Err(WebError::Middleware(
                    MiddleError::new(NoMatch, StatusCode::NotFound)
                )),
            }
        };

        let is_mounted = req.extensions.contains::<OriginalUrl>();
        if !is_mounted {
            req.extensions.insert::<OriginalUrl>(req.url.clone());
        }

        let path = req.url.path()[matched.length..].join("/");
        req.url.as_mut().set_path(&path);

        let res = matched.handler.handle(req);

        req.url = match req.extensions.get::<OriginalUrl>() {
            Some(url) => url.clone(),
            None => panic!("Original URL unexpectedly removed from extensions"),
        };
        
        if !is_mounted {
            req.extensions.remove::<OriginalUrl>();
        }

        res        
    }
}


