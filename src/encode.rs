extern crate hyper;
extern crate rustc_serialize;
extern crate core;

use hyper::server::{Request, Response};
use rustc_serialize::json;
use rustc_serialize::Encodable;
use std::io::Write;

// Wraps up one type of writer (hyper::net::streaming), which is given by the hyper server to write
// to, and exposes it as the type core::fmt::Write, which is expected by json::Encoder::new.
struct WriteWrap<'a> {
    res: Response<'a, hyper::net::Streaming>,
}

impl<'a> core::fmt::Write for WriteWrap<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Attempt to write to the response, and unwrap the error to make a fmt::Result instead of
        // an io::Result
        //
        if s.len() < 1 {
            return Ok(());
        }

        match self.res.write(s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(core::fmt::Error),
        }
    }
}

pub fn write_object<T: Encodable>(_: Request, res: Response, obj: &T) {
    // For now, only writes JSON, later, could read the Accept header
    let wrapped = &mut WriteWrap { res: res.start().unwrap() };

    {
        let enc = &mut json::Encoder::new(wrapped);
        obj.encode(enc).unwrap();
    }
    {
        use core::fmt::Write;
        wrapped.write_str("\n").unwrap();
    }
}
