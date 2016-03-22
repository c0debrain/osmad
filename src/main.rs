extern crate hyper;
extern crate rustc_serialize;
extern crate core;
extern crate rusqlite;
extern crate time;


use hyper::server::{Server, Request, Response};
use hyper::uri;
use rustc_serialize::json;
use rustc_serialize::Encodable;
use std::io::Write;
use std::convert::AsRef;
use std::sync::Mutex;
use time::Timespec;

use rusqlite::Connection;


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

fn write_object<T: Encodable>(_: Request, res: Response, obj: &T) {
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


#[derive(RustcDecodable, RustcEncodable)]
struct Timeslot {
    time: Timespec,
}

fn times_handler<'a>(req: Request, res: Response, conn: &Connection) {

    // Select all
    let mut stmt = conn.prepare("SELECT time FROM timeslot").unwrap();

    // Turn them into an iterator
    let timeslot_iter = stmt.query_map(&[], |row| Timeslot { time: row.get(0) })
                            .unwrap();

    // For now, convert the iterator into a vector, later, maybe we can encode the json on the fly?
    let mut timeslots: Vec<Timeslot> = Vec::new();
    for ts in timeslot_iter {
        timeslots.push(ts.unwrap());
    }
    write_object(req, res, &timeslots);
}



struct Handler {
    conn: Mutex<Connection>,
}

impl hyper::server::Handler for Handler {
    fn handle(&self, req: Request, res: Response) {
        let path = match req.uri {
            uri::RequestUri::AbsolutePath(ref path) => path.clone(),
            _ => return,
        };

        match path.as_ref() {
            "/times" => times_handler(req, res, &self.conn.lock().unwrap()),
            _ => return, // 404
        };

    }
}


fn main() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE timeslot(
     time TIMESTAMP NOT NULL PRIMARY KEY
     )",
                 &[])
        .unwrap();

    let t = Timeslot { time: time::get_time() };
    conn.execute("INSERT INTO timeslot (time) VALUES ($1)", &[&t.time]).unwrap();


    let handler = Handler { conn: Mutex::new(conn) };
    Server::http("0.0.0.0:8080").unwrap().handle(handler).unwrap();
}
