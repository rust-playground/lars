extern crate hyper;

use futures::Future;

use hyper::Error;
use hyper::{Request, Response};

use std::collections::HashMap;
use std::fmt;

pub trait Handler: 'static + Send + Sync {
    fn handle(
        &self,
        req: Request,
        params: PathParams,
    ) -> Box<Future<Item = Response, Error = Error>>;
}

impl fmt::Debug for Handler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "handler")
    }
}

impl<F> Handler for F
where
    F: 'static
        + Send
        + Sync
        + Fn(Request, PathParams) -> Box<Future<Item = Response, Error = Error>>,
{
    fn handle(
        &self,
        req: Request,
        params: PathParams,
    ) -> Box<Future<Item = Response, Error = Error>> {
        (*self)(req, params)
    }
}

pub struct Match<'a> {
    pub handler: &'a Box<Handler>,
    pub params: PathParams<'a>,
}

#[derive(Debug)]
pub struct PathParams<'p> {
    pub h: Option<HashMap<&'p str, String>>,
}

#[derive(Debug)]
pub struct NodeExtra {
    node: Box<Node>,
    id: String,
}

#[derive(Debug)]
pub struct Node {
    pub statics: Option<HashMap<String, Node>>,
    pub param: Option<NodeExtra>,
    pub wild: Option<NodeExtra>,
    pub handler: Option<Box<Handler>>,
}

impl Node {
    pub fn new() -> Node {
        return Node {
            statics: None,
            param: None,
            wild: None,
            handler: None,
        };
    }

    pub fn add(&mut self, path: &str, handler: Box<Handler>) {
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        let char = parts[0].chars().nth(0);

        if char == Some(':') {
            if self.param.is_none() {
                self.param = Some(NodeExtra {
                    node: Box::new(Node::new()),
                    id: parts[0].to_owned(),
                });
            } else {
                let id = &self.param.as_ref().unwrap().id;
                if id != parts[0] {
                    panic!(
                        "conflicting parameter id's detected '{}' and '{}' in remaining path '{}'",
                        id, parts[0], path
                    );
                }
            }
            let node = self.param.as_mut().unwrap().node.as_mut();
            if parts.len() == 1 {
                node.handler = Some(handler);
                return;
            }
            node.add(parts[1], handler)
        } else if char == Some('*') {
            // check is the rest of path has another wildcard or param, as that's not permitted.
            if parts.len() > 1 && (parts[1].contains("*") || parts[1].contains(":")) {
                panic!("no wildcard '*' nor param ':' is permitted after the first wildcard param, remaining path '{}'", path);
            }

            if self.wild.is_none() {
                self.wild = Some(NodeExtra {
                    node: Box::new(Node::new()),
                    id: parts[0].to_owned(),
                });
            } else {
                let id = &self.wild.as_ref().unwrap().id;
                if id != parts[0] {
                    panic!(
                        "conflicting wildcard id's detected '{}' and '{}' in remaining path '{}'",
                        id, parts[0], path
                    );
                }
            }
            self.wild.as_mut().unwrap().node.as_mut().handler = Some(handler);
            return;
        } else {
            if self.statics.is_none() {
                self.statics = Some(HashMap::new());
            }

            let node = self.statics
                .as_mut()
                .unwrap()
                .entry(parts[0].to_owned())
                .or_insert(Node::new());

            if parts.len() == 1 {
                node.handler = Some(handler);
                return;
            }
            node.add(parts[1], handler)
        }
    }

    pub fn find(&self, path: &str) -> Option<Match> {
        let parts: Vec<&str> = path.splitn(2, '/').collect();

        if self.statics.is_some() {
            let inner = self.statics.as_ref();
            if inner.is_some() {
                let node = inner.unwrap().get(parts[0]);

                if node.is_some() {
                    let node = node.unwrap();

                    if parts.len() == 1 {
                        let handler = &node.handler.as_ref()?;
                        return Some(Match {
                            handler,
                            params: PathParams { h: None },
                        });
                    }
                    return node.find(parts[1]);
                }
            }
        }

        if self.param.is_some() {
            let enode = self.param.as_ref().unwrap();
            let node = enode.node.as_ref();

            if parts.len() == 1 {
                if path.len() == 0 {
                    // can return none here as can't have a wildcard
                    // and a param at the same path
                    return None;
                }
                let handler = node.handler.as_ref()?;
                let mut m = Some(Match {
                    handler,
                    params: PathParams {
                        h: Some(HashMap::new()),
                    },
                });
                m.as_mut()
                    .unwrap()
                    .params
                    .h
                    .as_mut()
                    .unwrap()
                    .insert(enode.id.as_ref(), parts[0].to_owned());
                return m;
            }

            let mut results = node.find(parts[1]);
            if results.is_some() {
                let results = results.as_mut().unwrap();
                if results.params.h.is_none() {
                    results.params.h = Some(HashMap::new());
                }
                let p = results.params.h.as_mut().unwrap();
                p.insert(enode.id.as_ref(), parts[0].to_owned());
            }
            return results;
        }

        if self.wild.is_some() && path.len() > 0 {
            let enode = self.wild.as_ref().unwrap();
            let node = enode.node.as_ref();
            let handler = node.handler.as_ref()?;

            let mut m = Some(Match {
                handler,
                params: PathParams {
                    h: Some(HashMap::new()),
                },
            });
            m.as_mut()
                .unwrap()
                .params
                .h
                .as_mut()
                .unwrap()
                .insert(enode.id.as_ref(), path.to_owned());
            return m;
        }
        None
    }
}
