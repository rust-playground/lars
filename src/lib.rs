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
    extern crate tokio_core;

    use super::*;
    use futures::sync::oneshot::{self, Canceled, Sender};
    use futures::{Future, Stream};
    use hyper::header::UserAgent;
    use hyper::server::Http;
    use hyper::{Chunk, Client};
    use std::io;
    use std::str;
    use std::thread;
    use tests::tokio_core::reactor::Core;

    struct MW {}

    impl Middleware for MW {
        fn next(&self, handler: Box<Handler>) -> Box<Handler> {
            let func = move |req: Request, _params: PathParams| {
                let x = Box::new(handler.handle(req, _params).then(|mut f| {
                    f.as_mut().unwrap().set_status(hyper::BadRequest);
                    f
                }));
                let x: Box<Future<Item = Response, Error = hyper::Error>> = x;
                x
            };
            Box::new(func)
        }
    }

    fn test(req: Request, _params: PathParams) -> Box<Future<Item = Response, Error = Error>> {
        let body = format!("{}", req.uri());
        Box::new(futures::future::ok(
            Response::new()
                .with_status(StatusCode::Ok)
                .with_header(ContentLength(body.len() as u64))
                .with_body(body),
        ))
    }

    #[test]
    fn paths() {
        let (tx, rx) = oneshot::channel::<bool>();
        let finish = rx.and_then(|res| -> Result<(), Canceled> { Ok(()) })
            .map_err(|_| ());

        let h = thread::spawn(|| {
            let router = RouteBuilder::new()
                .get("/", test)
                .get("/test", test)
                .get("/test/:id/handler", test)
                .get("/test/:id/handler/*wild", test)
                .get_with_middleware("/with-middleware", test, vec![MW {}])
                .finalize();
            let addr = "127.0.0.1:3000".parse().unwrap();
            let server = Http::new().bind(&addr, router).unwrap();
            server.run_until(finish).unwrap();
        });

        let mut core = Core::new().unwrap();
        let client = Client::new(&core.handle());

        let work = client
            .get("http://localhost:3000/".parse().unwrap())
            .and_then(|res| {
                assert_eq!(res.status(), hyper::Ok);

                res.body().concat2().and_then(|body| {
                    let s = str::from_utf8(&body).unwrap();
                    assert_eq!("/", s);
                    Ok(())
                })
            });
        core.run(work).unwrap();

        let work = client
            .get("http://localhost:3000/test".parse().unwrap())
            .and_then(|res| {
                assert_eq!(res.status(), hyper::Ok);

                res.body().concat2().and_then(|body| {
                    let s = str::from_utf8(&body).unwrap();
                    assert_eq!("/test", s);
                    Ok(())
                })
            });
        core.run(work).unwrap();

        let work = client
            .get("http://localhost:3000/test/13/handler".parse().unwrap())
            .and_then(|res| {
                assert_eq!(res.status(), hyper::Ok);

                res.body().concat2().and_then(|body| {
                    let s = str::from_utf8(&body).unwrap();
                    assert_eq!("/test/13/handler", s);
                    Ok(())
                })
            });
        core.run(work).unwrap();

        let work = client
            .get("http://localhost:3000/test/".parse().unwrap())
            .and_then(|res| {
                assert_eq!(res.status(), hyper::NotFound);

                res.body().concat2().and_then(|body| {
                    let s = str::from_utf8(&body).unwrap();
                    assert_eq!("Not Found", s);
                    Ok(())
                })
            });
        core.run(work).unwrap();

        let work = client
            .get(
                "http://localhost:3000/test/13/handler/this/is/my/wildcard/path"
                    .parse()
                    .unwrap(),
            )
            .and_then(|res| {
                assert_eq!(res.status(), hyper::Ok);

                res.body().concat2().and_then(|body| {
                    let s = str::from_utf8(&body).unwrap();
                    assert_eq!("/test/13/handler/this/is/my/wildcard/path", s);
                    Ok(())
                })
            });
        core.run(work).unwrap();

        let work = client
            .get("http://localhost:3000/with-middleware".parse().unwrap())
            .and_then(|res| {
                assert_eq!(res.status(), hyper::BadRequest);

                res.body().concat2().and_then(|body| {
                    let s = str::from_utf8(&body).unwrap();
                    assert_eq!("/with-middleware", s);
                    Ok(())
                })
            });
        core.run(work).unwrap();

        drop(tx);
        let _ = h.join();
    }

    #[test]
    #[should_panic]
    fn panic_differing_param_path() {
        RouteBuilder::new()
            .get("/test/:id/handler", test)
            .get("/test/:user_id/handler/2", test);
    }

    #[test]
    #[should_panic]
    fn panic_wild_param_path() {
        RouteBuilder::new().get("/test/*wild/handler/:id", test);
    }

    #[test]
    #[should_panic]
    fn panic_differing_wild_path() {
        RouteBuilder::new()
            .get("/test/*wild", test)
            .get("/test/*wild2", test);
    }
}
