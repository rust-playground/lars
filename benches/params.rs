#![feature(test)]

extern crate futures;
extern crate hyper;
extern crate lars;
extern crate test;
extern crate tokio_core;

use test::Bencher;

use futures::Future;
use futures::sync::oneshot::{self, Canceled};
use hyper::header::ContentLength;
use hyper::server::Http;
use hyper::{Client, Error, Request, Response, StatusCode};
use lars::{RequestData, RouteBuilder};
use std::thread;
use tokio_core::reactor::Core;

fn test(req: Request, _params: RequestData) -> Box<Future<Item = Response, Error = Error>> {
    let body = format!("{}", req.uri());
    Box::new(futures::future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_header(ContentLength(body.len() as u64))
            .with_body(body),
    ))
}

#[bench]
fn bench_zero_params(b: &mut Bencher) {
    let (tx, rx) = oneshot::channel::<bool>();
    let finish = rx.and_then(|_res| -> Result<(), Canceled> { Ok(()) })
        .map_err(|_| ());

    let h = thread::spawn(|| {
        let router = RouteBuilder::new().get("/", test).finalize();
        let addr = "127.0.0.1:3000".parse().unwrap();
        let server = Http::new().bind(&addr, router).unwrap();
        server.run_until(finish).unwrap();
    });

    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    b.iter(|| {
        let work = client
            .get("http://localhost:3000/".parse().unwrap())
            .and_then(|_res| Ok(()));
        let _ = core.run(work);
    });

    drop(tx);
    let _ = h.join();
}

#[bench]
fn bench_two_params(b: &mut Bencher) {
    let (tx, rx) = oneshot::channel::<bool>();
    let finish = rx.and_then(|_res| -> Result<(), Canceled> { Ok(()) })
        .map_err(|_| ());

    let h = thread::spawn(|| {
        let router = RouteBuilder::new()
            .get("/user/:id1/address/:id2", test)
            .finalize();
        let addr = "127.0.0.1:3000".parse().unwrap();
        let server = Http::new().bind(&addr, router).unwrap();
        server.run_until(finish).unwrap();
    });

    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    b.iter(|| {
        let work = client
            .get("http://localhost:3000/user/13/address/1".parse().unwrap())
            .and_then(|_res| Ok(()));
        let _ = core.run(work);
    });

    drop(tx);
    let _ = h.join();
}

#[bench]
fn bench_five_params(b: &mut Bencher) {
    let (tx, rx) = oneshot::channel::<bool>();
    let finish = rx.and_then(|_res| -> Result<(), Canceled> { Ok(()) })
        .map_err(|_| ());

    let h = thread::spawn(|| {
        let router = RouteBuilder::new()
            .get(
                "/path1/:id1/path2/:id2/path3/:id3/path4/:id4/path5/:id5",
                test,
            )
            .finalize();
        let addr = "127.0.0.1:3000".parse().unwrap();
        let server = Http::new().bind(&addr, router).unwrap();
        server.run_until(finish).unwrap();
    });

    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    b.iter(|| {
        let work = client
            .get(
                "http://localhost:3000/path1/id1/path2/id2/path3/id3/path4/id4/path5/id5"
                    .parse()
                    .unwrap(),
            )
            .and_then(|_res| Ok(()));
        let _ = core.run(work);
    });

    drop(tx);
    let _ = h.join();
}

#[bench]
fn bench_eight_params(b: &mut Bencher) {
    let (tx, rx) = oneshot::channel::<bool>();
    let finish = rx.and_then(|_res| -> Result<(), Canceled> { Ok(()) })
        .map_err(|_| ());

    let h = thread::spawn(|| {
        let router = RouteBuilder::new()
            .get("/path1/:id1/path2/:id2/path3/:id3/path4/:id4/path5/:id5/path6/:id6/path7/:id7/path8/:id8", test)
            .finalize();
        let addr = "127.0.0.1:3000".parse().unwrap();
        let server = Http::new().bind(&addr, router).unwrap();
        server.run_until(finish).unwrap();
    });

    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    b.iter(|| {
        let work = client
            .get("http://localhost:3000/path1/id1/path2/id2/path3/id3/path4/id4/path5/id5/path6/id6/path7/id7/path8/id8".parse().unwrap())
            .and_then(|_res| Ok(()));
        let _ = core.run(work);
    });

    drop(tx);
    let _ = h.join();
}
