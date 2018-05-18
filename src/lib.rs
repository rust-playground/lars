extern crate futures;
extern crate hyper;
extern crate serde_json;

mod node;
mod router;

use hyper::Method;
use std::collections::HashMap;
use std::sync::Arc;

use node::Node;
use router::{Router, Routes};

pub use node::PathParams;

pub struct RouteBuilder {
    tree: Routes,
}

impl RouteBuilder {
    pub fn default() -> Self {
        RouteBuilder {
            tree: Routes::new(),
        }
    }

    pub fn add(mut self, method: Method, path: &str, handler: Box<node::Handler>) -> Self {
        self.tree
            .entry(method)
            .or_insert(Node::new())
            .add(path, handler);
        // TODO add route to node, must implement node add logic
        self
    }

    pub fn get(mut self, path: &str, handler: Box<node::Handler>) -> Self {
        self.add(hyper::Get, path, handler)
    }

    pub fn finalize(self) -> Router {
        Router::new(self.tree)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
