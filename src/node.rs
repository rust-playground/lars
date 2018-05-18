extern crate hyper;

use futures::Future;

use hyper::{Error, Method, StatusCode};
use hyper::{Request, Response};

use std::collections::HashMap;

pub trait Handler: 'static + Send + Sync {
    fn handle(
        &self,
        req: Request,
        params: PathParams,
    ) -> Box<Future<Item = Response, Error = Error>>;
}

impl<F> Handler for F
where
    F: 'static
        + Send
        + Sync
        + Fn(Request, PathParams) -> Box<Future<Item = Response, Error = Error>>,
{
    fn handle(
        &self,
        req: Request,
        params: PathParams,
    ) -> Box<Future<Item = Response, Error = Error>> {
        (*self)(req, params)
    }
}

pub struct PathParams {
    h: HashMap<String, String>,
}

pub struct Node {
    pub statics: Option<HashMap<String, Node>>,
    pub params: Option<HashMap<String, Node>>,
    pub wild: Option<HashMap<String, Node>>,
    pub param: Option<String>,
    pub handler: Option<Box<Handler>>,
}

impl Node {
    pub fn new() -> Node {
        return Node {
            statics: None,
            params: None,
            wild: None,
            param: None,
            handler: None,
        };
    }
    pub fn add(&mut self, path: &str, handler: Box<Handler>) {
        println!("{:?}", "HERE");
        let parts = path.split("/");
        for part in parts {
            println!("PART: {:?}", part);
        }
    }
}
