extern crate futures;
extern crate hyper;
extern crate serde_json;

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use futures::Future;
use hyper::header::{ContentLength, ContentType};
use hyper::server::{NewService, Request, Response, Service};
use hyper::{mime, Method};

use node::{Handler, Node};

pub type Routes = HashMap<Method, Node>;

pub struct Router {
    pub tree: Arc<Routes>,
}

impl Router {
    pub fn new(routes: Routes) -> Router {
        Router {
            tree: Arc::new(routes),
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
            tree: self.tree.clone(),
        })
    }
}

pub struct RouterService {
    tree: Arc<Routes>,
}

impl Service for RouterService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let uri = format!("{:?}", req.uri());
        let json = serde_json::to_string(&uri).unwrap();
        Box::new(futures::future::ok(
            Response::new()
                .with_header(ContentType(mime::APPLICATION_JSON))
                .with_header(ContentLength(json.len() as u64))
                .with_body(json),
        ))
        //        let method = req.method().clone();
        //        let path = req.path().to_owned();
        //        match self.inner.recognize(&method, &path) {
        //            Ok((handler, cap)) => handler.handle(req, cap),
        //            Err(code) => futures::future::ok(Response::new().with_status(code)).boxed(),
        //        }
    }
}
