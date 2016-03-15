extern crate hyper;

use hyper::server::{Server, Request, Response};
use std::io::Write;

fn hello(req: Request, res: Response) {
    res.start().unwrap().write("Hälló, wørld\n".as_bytes());
}


fn main() {
    Server::http("0.0.0.0:8080").unwrap().handle(hello).unwrap();
}
