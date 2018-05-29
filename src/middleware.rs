use node::Handler;

/// Middleware allows for additional handlers to wrap the request.
///
/// The request will run through these Handlers before proceeding to the next and also allow for handling the response on the way out.
/// Each Middleware is run i nthe order which it is registered.
///
/// # Examples
/// ```
/// extern crate futures;
/// extern crate hyper;
/// extern crate lars;
///
/// use lars::{Handler, RequestData, RouteBuilder, Middleware};
/// use hyper::{Error, Request, Response, StatusCode};
/// use futures::{Future, future};
/// use hyper::server::Http;
///
/// let router = RouteBuilder::new()
///     .with_middleware(MyMiddleware{})
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
///
/// struct MyMiddleware {}
///
/// impl Middleware for MyMiddleware {
///    fn next(&self, handler: Box<Handler>) -> Box<Handler> {
///        let func = move |req: Request, data: RequestData| {
///            println!("BEFORE");
///            let x = Box::new(handler.handle(req, data).then(|f| {
///                println!("AFTER {:?}", f);
///                f
///            }));
///            let x: Box<Future<Item = Response, Error = hyper::Error>> = x;
///           x
///        };
///
///        Box::new(func)
///    }
/// }
/// ```
pub trait Middleware {
    fn next(&self, next: Box<Handler>) -> Box<Handler>;
}

impl<F> Middleware for F
where
    F: 'static + Send + Sync + Fn(Box<Handler>) -> Box<Handler>,
{
    fn next(&self, handler: Box<Handler>) -> Box<Handler> {
        (*self)(handler)
    }
}
