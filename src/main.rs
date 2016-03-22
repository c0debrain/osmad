extern crate hyper;
extern crate rustc_serialize;
extern crate core;
extern crate rusqlite;
extern crate time;


use hyper::server::{Server, Request, Response};
use hyper::uri;
use std::convert::AsRef;
use std::sync::Mutex;
use time::{Tm, TmFmt, Duration};

use rusqlite::Connection;

mod encode;


#[derive(RustcDecodable, RustcEncodable)]
struct Timeslot {
    time: String,
}

fn times_handler<'a>(req: Request, res: Response, conn: &Connection) {

    // Select all
    let mut stmt = conn.prepare("SELECT time FROM timeslot").unwrap();

    // Turn them into an iterator
    let timeslot_iter = stmt.query_map(&[], |row| {
                                Timeslot { time: format!("{}", time::at(row.get(0)).rfc3339()) }
                            })
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

struct TimeIterator {
    end: time::Tm,
    interval: time::Duration,
    current: time::Tm,
}

impl TimeIterator {
    fn new(start: Tm, end: Tm, interval: time::Duration) -> TimeIterator {
        return TimeIterator {
            end: end,
            current: start - interval,
            interval: interval,
        };
    }
}

impl Iterator for TimeIterator {
    type Item = time::Tm;
    fn next(&mut self) -> Option<time::Tm> {
        self.current = self.current + self.interval;
        if self.current >= self.end {
            None
        } else {
            Some(self.current)
        }
    }
}

const RFC3339: &'static str = "%FT%T%z";

fn main() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE timeslot(
     time TIMESTAMP NOT NULL PRIMARY KEY
     )",
                 &[])
        .unwrap();

    for t in TimeIterator::new(time::strptime("2016-01-01T07:00:00+11:00", RFC3339).unwrap(),
                               time::strptime("2016-01-01T09:00:00+11:00", RFC3339).unwrap(),
                               Duration::minutes(6)) {

        conn.execute("INSERT INTO timeslot (time) VALUES ($1)",
                     &[&t.to_timespec()])
            .unwrap();
    }

    let handler = Handler { conn: Mutex::new(conn) };
    Server::http("0.0.0.0:8080").unwrap().handle(handler).unwrap();
}
