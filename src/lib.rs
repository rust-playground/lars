extern crate futures;
extern crate hyper;

mod middleware;
mod node;
mod router;

use hyper::header::{Allow, ContentLength};
use hyper::server::{Request, Response};
use hyper::{Error, Method, StatusCode};

use futures::Future;

use node::Node;
use router::{Router, Routes};

use std::collections::HashMap;

pub use middleware::Middleware;
pub use node::{Handler, RequestData};

pub struct RouteBuilder {
    tree: Routes,
    not_found: Box<node::Handler>,
    middleware: Option<Vec<Box<Middleware>>>,
}

impl RouteBuilder {
    /// Constructs a new `RouteBuilder`.
    ///
    /// The `RouteBuilder` is used to register the routes, middleware and set custom values.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate futures;
    /// extern crate hyper;
    /// extern crate lars;
    ///
    /// use lars::{Handler, RequestData, RouteBuilder};
    /// use hyper::{Error, Request, Response, StatusCode};
    /// use futures::{Future, future};
    /// use hyper::server::Http;
    ///
    /// let router = RouteBuilder::new()
    ///     .get("/", root)
    ///     .finalize();
    ///
    /// let addr = "127.0.0.1:3000".parse().unwrap();
    /// let server = Http::new().bind(&addr, router).unwrap();
    /// // server.run().unwrap();
    ///
    /// fn root(req: Request, data: RequestData) -> Box<Future<Item = Response, Error = Error>> {
    ///    Box::new(future::ok(
    ///        Response::new()
    ///            .with_status(StatusCode::Ok)
    ///            .with_body("root"),
    ///    ))
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// - If two similar routes are configured with differing parameter names eg. `/user/:foo` and `/user/:bar/profile`
    /// - If registering a duplicate wildcard route with differing wildcard names eg. `/user/*foo` and `/user/*bar`
    /// - If a parameter or wildcard is configured after a wildcard eg. `/user/*/:foo`
    ///
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

        let mut params: HashMap<&str, bool> = HashMap::new();
        for p in path.split("/").collect::<Vec<&str>>() {
            if p.chars().nth(0) == Some(':') {
                if params.get(p).is_some() {
                    panic!(
                        "conflicting parameter names detected for path {}, for paramter {}",
                        path, p
                    )
                }
                params.insert(p, true);
            }
        }

        let mut h: Box<node::Handler> = Box::new(handler);

        // middleware just for this handler
        if middleware.is_some() {
            let mw = middleware.as_ref().unwrap();
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
        let find = Find::new(self.tree, self.not_found);
        let mut h: Box<node::Handler> = Box::new(find);

        // global middleware
        if self.middleware.is_some() {
            let mw = self.middleware.as_ref().unwrap();
            for i in (0..mw.len()).rev() {
                h = mw[i].next(h);
            }
        }
        Router::new(h)
    }
}

struct Find {
    tree: Routes,
    not_found: Box<node::Handler>,
}

impl Find {
    pub fn new(tree: Routes, not_found: Box<Handler>) -> Self {
        Find { tree, not_found }
    }
}

impl Handler for Find {
    fn handle(
        &self,
        req: Request,
        _params: RequestData,
    ) -> Box<Future<Item = Response, Error = Error>> {
        let p = req.path().to_owned();
        let (_, right) = p.split_at(1);

        let node = self.tree.get(req.method());
        if node.is_none() {
            return handle_method_not_allowed_not_found(&self.tree, &self.not_found, req, right);
        }
        let m = node.unwrap().find(right);
        if m.is_none() {
            return handle_method_not_allowed_not_found(&self.tree, &self.not_found, req, right);
        }
        let m = m.unwrap();
        m.handler.handle(req, m.params)
    }
}

fn handle_method_not_allowed_not_found(
    tree: &Routes,
    not_found: &Box<node::Handler>,
    req: Request,
    path: &str,
) -> Box<Future<Item = Response, Error = hyper::Error>> {
    const METHOD_NOT_ALLOWED: &'static str = "Method Not Allowed";
    let mut found = false;
    let mut methods: Vec<Method> = Vec::new();

    for (k, v) in tree {
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
    not_found.handle(req, RequestData { params: None })
}

const NOT_FOUND: &'static str = "Not Found";

fn not_found(_req: Request, _params: RequestData) -> Box<Future<Item = Response, Error = Error>> {
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

    use futures::sync::oneshot::{self, Canceled};
    use futures::{Future, Stream};
    use hyper::Client;
    use hyper::server::Http;
    use std::str;
    use std::thread;
    use tests::tokio_core::reactor::Core;

    struct MW {}

    impl Middleware for MW {
        fn next(&self, handler: Box<Handler>) -> Box<Handler> {
            let func = move |req: Request, _params: RequestData| {
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

    fn test(req: Request, _params: RequestData) -> Box<Future<Item = Response, Error = Error>> {
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
        let finish = rx.and_then(|_res| -> Result<(), Canceled> { Ok(()) })
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

    #[test]
    #[should_panic]
    fn panic_duplicate_param_names_path() {
        RouteBuilder::new().get("/test/:id/handler/:id", test);
    }

    #[test]
    #[should_panic]
    fn panic_duplicate_wild_names_path() {
        RouteBuilder::new().get("/test/*id/handler/*id", test);
    }
}
