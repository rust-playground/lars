use node::Handler;

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
