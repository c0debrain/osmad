extern crate hyper;
extern crate rustc_serialize;
extern crate core;

use hyper::server::{Server, Request, Response};
use rustc_serialize::json;
use rustc_serialize::Encodable;
use std::io::Write;

#[derive(RustcDecodable, RustcEncodable)]
struct Hello {
    greeting: String,
}

// Wraps up one type of writer (hyper::net::streaming), which is given by the hyper server to write
// to, and exposes it as the type core::fmt::Write, which is expected by json::Encoder::new.
struct WriteWrap<'a> {
    res: Response<'a, hyper::net::Streaming>,
}

impl<'a> core::fmt::Write for WriteWrap<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Attempt to write to the response, and unwrap the error to make a fmt::Result instead of
        // an io::Result
        match self.res.write(s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(core::fmt::Error),
        }
    }
}

fn hello(req: Request, res: Response) {
    let hw = Hello { greeting: "Hälló, wørld".to_string() };

    // Create a new WriteWrapper for the response
    let wrapped = &mut WriteWrap { res: res.start().unwrap() };

    // Create a new json::Encoder with the wrapped writer
    let enc = &mut json::Encoder::new(wrapped);

    // The [RustcEncodable] trait adds the 'encode' method to the hw struct.
    hw.encode(enc).unwrap();
}


fn main() {
    Server::http("0.0.0.0:8080").unwrap().handle(hello).unwrap();
}
