extern crate futures;
extern crate hyper;

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use futures::Future;
use hyper::Method;
use hyper::server::{NewService, Request, Response, Service};

use node::{Handler, Node, RequestData};

pub type Routes = HashMap<Method, Node>;

pub struct Router {
    pub handler: Arc<Box<Handler>>,
}

impl Router {
    pub fn new(handler: Box<Handler>) -> Router {
        Router {
            handler: Arc::new(handler),
        }
    }
}

impl NewService for Router {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Instance = RouterService;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(RouterService {
            handler: self.handler.clone(),
        })
    }
}

pub struct RouterService {
    handler: Arc<Box<Handler>>,
}

impl Service for RouterService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Response, Error = hyper::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        self.handler.handle(req, RequestData { params: None })
    }
}
