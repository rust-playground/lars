extern crate futures;
extern crate hyper;

mod middleware;
mod node;
mod router;

use hyper::header::ContentLength;
use hyper::server::{Request, Response};
use hyper::{Error, Method, StatusCode};

use futures::Future;

use node::Node;
use router::{Router, Routes};

pub use middleware::Middleware;
pub use node::{Handler, PathParams};

pub struct RouteBuilder {
    tree: Routes,
    not_found: Box<node::Handler>,
    middleware: Option<Vec<Box<Middleware>>>,
}

impl RouteBuilder {
    pub fn new() -> Self {
        RouteBuilder {
            tree: Routes::new(),
            not_found: Box::new(not_found),
            middleware: None,
        }
    }

    pub fn with_middleware<MW>(mut self, middleware: MW) -> Self
    where
        MW: Sized + Middleware + 'static,
    {
        if self.middleware.is_none() {
            self.middleware = Some(Vec::new());
        }
        self.middleware.as_mut().unwrap().push(Box::new(middleware));
        self
    }

    pub fn with_middlewares<MW>(mut self, middleware: Vec<MW>) -> Self
    where
        MW: Sized + Middleware + 'static,
    {
        if self.middleware.is_none() {
            self.middleware = Some(Vec::new());
        }
        {
            let mw = self.middleware.as_mut().unwrap();
            for m in middleware {
                mw.push(Box::new(m));
            }
        }
        self
    }

    pub fn set_not_found<H>(mut self, handler: H) -> Self
    where
        H: Sized + node::Handler,
    {
        self.not_found = Box::new(handler);
        self
    }

    pub fn add_with_middleware<H>(
        mut self,
        method: Method,
        path: &str,
        handler: H,
        middleware: Option<Vec<Box<Middleware>>>,
    ) -> Self
    where
        H: Sized + node::Handler,
    {
        let (left, right) = path.split_at(1);

        if left != "/" {
            panic!("paths must start with '/'");
        }

        let mut h: Box<node::Handler> = Box::new(handler);

        // middleware just for this handler
        if middleware.is_some() {
            let mw = middleware.as_ref().unwrap();
            for i in (0..mw.len()).rev() {
                h = mw[i].next(h);
            }
        }

        // global middleware
        if self.middleware.is_some() {
            let mw = self.middleware.as_ref().unwrap();
            for i in (0..mw.len()).rev() {
                h = mw[i].next(h);
            }
        }

        self.tree.entry(method).or_insert(Node::new()).add(right, h);
        self
    }

    pub fn add<H>(self, method: Method, path: &str, handler: H) -> Self
    where
        H: Sized + node::Handler,
    {
        self.add_with_middleware(method, path, handler, None)
    }

    pub fn get<H>(self, path: &str, handler: H) -> Self
    where
        H: Sized + node::Handler,
    {
        self.add(Method::Get, path, handler)
    }

    pub fn get_with_middleware<H, MW>(self, path: &str, handler: H, middleware: Vec<MW>) -> Self
    where
        H: Sized + node::Handler,
        MW: Sized + Middleware + 'static,
    {
        let mut mw: Vec<Box<Middleware>> = Vec::new();
        for m in middleware {
            mw.push(Box::new(m));
        }
        self.add_with_middleware(Method::Get, path, handler, Some(mw))
    }

    pub fn head<H>(self, path: &str, handler: H) -> Self
    where
        H: Sized + node::Handler,
    {
        self.add(Method::Head, path, handler)
    }

    pub fn head_with_middleware<H, MW>(self, path: &str, handler: H, middleware: Vec<MW>) -> Self
    where
        H: Sized + node::Handler,
        MW: Sized + Middleware + 'static,
    {
        let mut mw: Vec<Box<Middleware>> = Vec::new();
        for m in middleware {
            mw.push(Box::new(m));
        }
        self.add_with_middleware(Method::Head, path, handler, Some(mw))
    }

    pub fn post<H>(self, path: &str, handler: H) -> Self
    where
        H: Sized + node::Handler,
    {
        self.add(Method::Post, path, handler)
    }

    pub fn post_with_middleware<H, MW>(self, path: &str, handler: H, middleware: Vec<MW>) -> Self
    where
        H: Sized + node::Handler,
        MW: Sized + Middleware + 'static,
    {
        let mut mw: Vec<Box<Middleware>> = Vec::new();
        for m in middleware {
            mw.push(Box::new(m));
        }
        self.add_with_middleware(Method::Post, path, handler, Some(mw))
    }

    pub fn put<H>(self, path: &str, handler: H) -> Self
    where
        H: Sized + node::Handler,
    {
        self.add(Method::Put, path, handler)
    }

    pub fn put_with_middleware<H, MW>(self, path: &str, handler: H, middleware: Vec<MW>) -> Self
    where
        H: Sized + node::Handler,
        MW: Sized + Middleware + 'static,
    {
        let mut mw: Vec<Box<Middleware>> = Vec::new();
        for m in middleware {
            mw.push(Box::new(m));
        }
        self.add_with_middleware(Method::Put, path, handler, Some(mw))
    }

    pub fn delete<H>(self, path: &str, handler: H) -> Self
    where
        H: Sized + node::Handler,
    {
        self.add(Method::Delete, path, handler)
    }

    pub fn delete_with_middleware<H, MW>(self, path: &str, handler: H, middleware: Vec<MW>) -> Self
    where
        H: Sized + node::Handler,
        MW: Sized + Middleware + 'static,
    {
        let mut mw: Vec<Box<Middleware>> = Vec::new();
        for m in middleware {
            mw.push(Box::new(m));
        }
        self.add_with_middleware(Method::Delete, path, handler, Some(mw))
    }

    pub fn connect<H>(self, path: &str, handler: H) -> Self
    where
        H: Sized + node::Handler,
    {
        self.add(Method::Connect, path, handler)
    }

    pub fn connect_with_middleware<H, MW>(self, path: &str, handler: H, middleware: Vec<MW>) -> Self
    where
        H: Sized + node::Handler,
        MW: Sized + Middleware + 'static,
    {
        let mut mw: Vec<Box<Middleware>> = Vec::new();
        for m in middleware {
            mw.push(Box::new(m));
        }
        self.add_with_middleware(Method::Connect, path, handler, Some(mw))
    }

    pub fn options<H>(self, path: &str, handler: H) -> Self
    where
        H: Sized + node::Handler,
    {
        self.add(Method::Options, path, handler)
    }

    pub fn options_with_middleware<H, MW>(self, path: &str, handler: H, middleware: Vec<MW>) -> Self
    where
        H: Sized + node::Handler,
        MW: Sized + Middleware + 'static,
    {
        let mut mw: Vec<Box<Middleware>> = Vec::new();
        for m in middleware {
            mw.push(Box::new(m));
        }
        self.add_with_middleware(Method::Options, path, handler, Some(mw))
    }

    pub fn trace<H>(self, path: &str, handler: H) -> Self
    where
        H: Sized + node::Handler,
    {
        self.add(Method::Trace, path, handler)
    }

    pub fn trace_with_middleware<H, MW>(self, path: &str, handler: H, middleware: Vec<MW>) -> Self
    where
        H: Sized + node::Handler,
        MW: Sized + Middleware + 'static,
    {
        let mut mw: Vec<Box<Middleware>> = Vec::new();
        for m in middleware {
            mw.push(Box::new(m));
        }
        self.add_with_middleware(Method::Trace, path, handler, Some(mw))
    }

    pub fn patch<H>(self, path: &str, handler: H) -> Self
    where
        H: Sized + node::Handler,
    {
        self.add(Method::Patch, path, handler)
    }

    pub fn patch_with_middleware<H, MW>(self, path: &str, handler: H, middleware: Vec<MW>) -> Self
    where
        H: Sized + node::Handler,
        MW: Sized + Middleware + 'static,
    {
        let mut mw: Vec<Box<Middleware>> = Vec::new();
        for m in middleware {
            mw.push(Box::new(m));
        }
        self.add_with_middleware(Method::Patch, path, handler, Some(mw))
    }

    pub fn finalize(self) -> Router {
        Router::new(self.tree, self.not_found)
    }
}

const NOT_FOUND: &'static str = "Not Found";

fn not_found(_req: Request, _params: PathParams) -> Box<Future<Item = Response, Error = Error>> {
    Box::new(futures::future::ok(
        Response::new()
            .with_status(StatusCode::NotFound)
            .with_header(ContentLength(NOT_FOUND.len() as u64))
            .with_body(NOT_FOUND),
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
