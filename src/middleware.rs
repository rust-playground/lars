use node::Handler;

pub trait Middleware {
    fn next(&self, next: Box<Handler>) -> Box<Handler>;
}
