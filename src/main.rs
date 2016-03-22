extern crate hyper;
extern crate rustc_serialize;
extern crate core;
extern crate rusqlite;
extern crate time;


use hyper::server::{Server, Request, Response};
use hyper::uri;
use std::convert::AsRef;
use std::sync::Mutex;
use time::Timespec;

use rusqlite::Connection;

mod encode;


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
    encode::write_object(req, res, &timeslots);
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
