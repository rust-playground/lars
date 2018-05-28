extern crate futures;
extern crate hyper;

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use futures::Future;
use hyper::header::{Allow, ContentLength};
use hyper::server::{NewService, Request, Response, Service};
use hyper::{Method, StatusCode};

use node::{Handler, Node, PathParams};

pub type Routes = HashMap<Method, Node>;

pub struct RouterConfig {
    tree: Routes,
    not_found: Box<Handler>,
}

pub struct Router {
    pub config: Arc<RouterConfig>,
}

impl Router {
    pub fn new(routes: Routes, not_found: Box<Handler>) -> Router {
        Router {
            config: Arc::new(RouterConfig {
                tree: routes,
                not_found,
            }),
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
            config: self.config.clone(),
        })
    }
}

pub struct RouterService {
    config: Arc<RouterConfig>,
}

impl Service for RouterService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Response, Error = hyper::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let p = req.path().to_owned();
        let (_, right) = p.split_at(1);

        let node = &self.config.tree.get(req.method());
        if node.is_none() {
            return handle_method_not_allowed_not_found(self, req, right);
        }
        let m = node.unwrap().find(right);
        if m.is_none() {
            return handle_method_not_allowed_not_found(self, req, right);
        }
        let m = m.unwrap();
        m.handler.handle(req, m.params)
    }
}

fn handle_method_not_allowed_not_found(
    rs: &RouterService,
    req: Request,
    path: &str,
) -> Box<Future<Item = Response, Error = hyper::Error>> {
    const METHOD_NOT_ALLOWED: &'static str = "Method Not Allowed";
    let mut found = false;
    let mut methods: Vec<Method> = Vec::new();

    for (k, v) in &rs.config.tree {
        if k == req.method() {
            continue;
        }
        let m = v.find(path);
        if m.is_some() {
            methods.push(k.clone());
            found = true;
        }
    }
    if found {
        return Box::new(futures::future::ok(
            Response::new()
                .with_status(StatusCode::MethodNotAllowed)
                .with_header(ContentLength(METHOD_NOT_ALLOWED.len() as u64))
                .with_header(Allow(methods))
                .with_body(METHOD_NOT_ALLOWED),
        ));
    }
    rs.config.not_found.handle(req, PathParams { params: None })
}
