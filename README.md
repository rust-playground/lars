# lars

lars, short for Library Access Retrieval System, is an HTTP router for [Hyper 11.x](https://github.com/hyperium/hyper) that supports 
parameters and wildcards in the URL as well as middleware.

- [x] Supports dynamic parameters that start with a colon `:`, eg. `/users/:id`
- [x] Supports wildcard routes and captures the remaining path, eg. `/static/*`
- [x] Support middleware, both defined at the global level and per route!

Usage
-----
First, add this to your `Cargo.toml`:
```toml
[dependencies]
lars = "0.0.1"
```

Here is a quite example to get you going:
```rust
extern crate futures;
extern crate hyper;
extern crate lars;

use futures::Future;

use hyper::header::ContentLength;
use hyper::server::Http;
use hyper::{Error, Request, Response, StatusCode};

use lars::{Handler, Middleware, RequestData, RouteBuilder};

fn main() {
    let mw = MyMiddleware {};
    let router = RouteBuilder::new()
        .with_middleware(static_middleware)
        .with_middleware(mw)
        .set_not_found(custom_not_found)
        .get("/", home)
        .get("/static/*path", static_resources)
        .get("/test/:id/handler", test_handler)
        .finalize();
    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new().bind(&addr, router).unwrap();
    server.run().unwrap();
}

struct MyMiddleware {}

impl Middleware for MyMiddleware {
    fn next(&self, handler: Box<Handler>) -> Box<Handler> {
        let func = move |req: Request, _params: RequestData| {
            println!("MyMiddleware BEFORE");
            let x = Box::new(handler.handle(req, _params).then(|f| {
                println!("MyMiddleware AFTER {:?}", f);
                f
            }));
            let x: Box<Future<Item = Response, Error = hyper::Error>> = x;
            x
        };
        Box::new(func)
    }
}

fn static_middleware(handler: Box<Handler>) -> Box<Handler> {
    let func = move |req: Request, _params: RequestData| {
        println!("Static Middleware BEFORE");
        let x = Box::new(handler.handle(req, _params).then(|f| {
            println!("Static Middleware AFTER {:?}", f);
            f
        }));
        let x: Box<Future<Item = Response, Error = hyper::Error>> = x;
        x
    };

    Box::new(func)
}

fn home(_req: Request, params: RequestData) -> Box<Future<Item = Response, Error = Error>> {
    let body = format!("Hello, world {:?}", params);
    Box::new(futures::future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_header(ContentLength(body.len() as u64))
            .with_body(body),
    ))
}

fn static_resources(
    _req: Request,
    params: RequestData,
) -> Box<Future<Item = Response, Error = Error>> {
    let body = format!("static resources {:?}", params);
    Box::new(futures::future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_header(ContentLength(body.len() as u64))
            .with_body(body),
    ))
}

fn test_handler(_req: Request, params: RequestData) -> Box<Future<Item = Response, Error = Error>> {
    let body = format!("test handler {:?}", params);
    Box::new(futures::future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_header(ContentLength(body.len() as u64))
            .with_body(body),
    ))
}

fn custom_not_found(
    _req: Request,
    params: RequestData,
) -> Box<Future<Item = Response, Error = Error>> {
    let body = format!("custom not found {:?}", params);
    Box::new(futures::future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_header(ContentLength(body.len() as u64))
            .with_body(body),
    ))
}
```