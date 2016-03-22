# Rust - From 0 to ... works.

```bash
cargo new osmad
```

Oh, no, right, that's for a library.

```bash
rm -rf osmad
cargo new --bin osmad
cd osmad

cargo run
   Compiling osmad v0.1.0 (file:///home/daemonl/learn/osmad)
     Running `target/debug/osmad`
Hello, world!
```

Hello world done. Hometime.

OSMaD is a real theatre company in Melbourne, Australia [www.osmad.com.au]. They have one show per year in about November. Audition bookings open in May, and for the last few years, those have been booked through an online 'booking system'.

It started when I lost the code for the first one. Well, it was PHP anyway. Then I re-wrote it in node. Node was ok, but the next year I couldn't quite remember all of the dependencies (no Docker back then) so I re-wrote it. Now it's just a habit. Each year, I write it in a new language or framework. Gives me a useful project, and a deadline.

So far it has been PHP (Symfony2), Node.js, Java and Go. Time for a challenge this year, I'll give Rust a shot.

Rust seems... Interesting. I doubt I will ever work on a project which actually requires it, I write APIs and integration things, usually the bottleneck is with some other component I didn't write, and time-to-market wins over memory footprint or a 'GC pause'. But hey, practicality isn't the point here. Learning is.

git tag v0.0.0

Step 1: HTTP
============

I'm going to use hyper, which means adding it as a dependency.
Rust dependencies are managed in to Cargo.toml file, kind of like package.json. Just add to the dependencies section:

```toml
[dependencies]
hyper = "*"
```
and next time you run 'cargo run', it will download and install hyper.


Then I'm going to pop in some example code from the Hyper website.

```rust
use hyper::server::{Server, Request, Response};

fn hello(req: Request, res: Response) {
    // handle things here
}

fn main() {
    Server::http("0.0.0.0:0").unwrap().handle(hello).unwrap();
}
```
```bash
$ cargo run
   Compiling osmad v0.1.0 (file:///home/daemonl/learn/osmad)
src/main.rs:1:21: 1:26 error: unresolved import `hyper::server::Server`. Maybe a missing `extern crate hyper`? [E0432]
src/main.rs:1 use hyper::server::{Server, Request, Response};
```
I already have a love-hate relationship with this compiler. It's always cheery and helpful. Right now it's telling me exactly what I missed.

To add a dependency:

- Include it in the Cargo.toml file
- Include a marker in the .rs file you need it
- 'use' the parts of it you want.

```rust
extern crate hyper;

use hyper::server::{Server, Request, Response};
...
```

Now it runs, giving warnings about the unused things.

Time for hello world again!

```rust
fn hello(req: Request, res: Response) {
    res.start().unwrap().write("Hälló, wørld\n".as_bytes());
}
```
What I think that does:

- Open up the response writer
- Unwrap - IO commands return an 'IO Result' - the unwrap makes it panic for any errors, and return the gooey insides, in this case, a hyper::net::Streaming - something which can be written to.
- Write the string "Hälló, wørld" and a newline, as UTF-8

But there is a problem:

```bash
src/main.rs:7:26: 7:58 help: items from traits can only be used if the trait is in scope; the following trait is implemented but not in scope, perhaps add a `use` for it:
src/main.rs:7:26: 7:58 help: candidate #1: use `std::io::Write`
```

I haven't exactly got my head around the 'why' for this one yet - it has come up a few times, but it looks like because I'm using 'write' directly, which is actually part of the std::io::Write 'trait' (read: 'interface'), even though I am not actually referring to the trait name anywhere, I still need to import it. Actually, I have always thought this a little odd in Java or go, you can use things without importing them, so long as you don't name them.

The fix is easy, do exactly what the message says and add 'use std::io::Write' to the top.

```bash
$ curl localhost:8080
Hälló, wørld
```

git tag v0.0.1


Step 2: JSON
============

This is going to be a JSON API server. This step will encode a struct as JSON and send it.

There is some work on 'better' json libraries than the 'default' one, but I am just going to use rustc_serialize here.

Refresher, adding a dependency:

- Include it in the Cargo.toml file
- Include a marker in the .rs file you need it
- 'use' the parts of it you want.

Oh, but wait, a gotcha here - the rustc_serialize is added as rustc-serialize. But only in Cargo.toml. (_ vs -)

```rust
extern crate hyper;
extern crate rustc_serialize;

use hyper::server::{Server, Request, Response};
use std::io::Write;
use rustc_serialize::json;

#[derive(RustcDecodable, RustcEncodable)]
struct Hello {
    greeting: String,
}

fn hello(req: Request, res: Response) {
    let hw = Hello { greeting: "Hälló, wørld".to_string() };
    let encoded = json::encode(&hw).unwrap();
    res.start().unwrap().write(encoded.as_bytes());
}


fn main() {
    Server::http("0.0.0.0:8080").unwrap().handle(hello).unwrap();
}
```

A few things here.

The derive tags tell rust to 'automatically derive' the Encodeable and Decodable traits. I have no idea how that works. But it does, for now. What it means for now is that the json::encode call works for that struct.

The '.to_string()'. Isn't it already a String? well, no. Try it without, the compiler will tell you it's a "&'static str" - that is, a borrowable, static reference to a 'str' type. Or something. For now, I'm just copying the example in [the docs](https://doc.rust-lang.org/rustc-serialize/rustc_serialize/json/index.html).

```bash
$ curl localhost:8080
{"greeting":"Hälló, wørld"}
```

It seems odd that we are converting to a string, then to bytes, then writing the bytes to the writer - At least after node and go - can we encode directly to the writer?

[Not Easily](https://github.com/rust-lang-nursery/rustc-serialize/pull/125), but we aren't about ease, right?

The basic problem is that the hyper response is an std::io::Write, but the json encoder expects a core::fmt::Write.

std::io::Write exposes:

```rust
fn write(&mut self, buf: &[u8]) -> Result<usize>
```

core::fmt::Write requires:

```rust
fn write_str(&mut self, s: &str) -> Result
```

To let's write an adaptor!

We need a struct which wraps the writer we have: std, but looks like the writer we need: core.


```rust
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
```

I have no idea if that was actually more efficient. Gut feel?

git tag v0.0.2

Step 3. Shooting the messenger
==============================

The server so far only has one path. Easy, with matchers and such, this should be easy as pie.

First thing to note, the request URI is an enum of various types. We (I think) only care about AbsolutePath (TODO: Read up on the other types), so we will only match one.

A cool thing is that we can return from within a match - I wasn't expecting that, so that's cool.

And thirdly, path.as_ref() - see [stack overflow](http://stackoverflow.com/questions/25383488/how-to-match-a-string-in-rust)


```rust
fn handler(req: Request, res: Response) {
    let path = match req.uri {
        uri::RequestUri::AbsolutePath(path) => path,
        _ => return,
    };
    let hw = match path.as_ref() {
        "/" => helloHandler(req),
        _ => return, // 404
    };
    //Do things...
}
```

However:

src/main.rs:46:29: 46:32 error: use of partially moved value: `req` [E0382]
src/main.rs:46         "/" => helloHandler(req),
                                           ^~~
src/main.rs:46:29: 46:32 help: run `rustc --explain E0382` to see a detailed explanation
src/main.rs:40:9: 40:12 note: `req.uri` moved here because it has type `hyper::uri::RequestUri`, which is moved by default
src/main.rs:40     let uri = req.uri;
                       ^~~
src/main.rs:40:9: 40:12 help: if you would like to borrow the value instead, use a `ref` binding as shown:
src/main.rs:       let ref uri = req.uri;

From the rust docs:

> However, this system does have a certain cost: learning curve. Many new users
> to Rust experience something we like to call ‘fighting with the borrow
> checker’, where the Rust compiler refuses to compile a program that the
> author thinks is valid. This often happens because the programmer's mental
> model of how ownership should work doesn't match the actual rules that Rust
> implements. You probably will experience similar things at first. There is
> good news, however: more experienced Rust developers report that once they
> work with the rules of the ownership system for a period of time, they fight
> the borrow checker less and less.

To fix it in this case, reading the hints and many documentations, make the
matcher match a 'ref', and clone it for the path.

```rust
    let path = match req.uri {
        uri::RequestUri::AbsolutePath(ref path) => path.clone(),
        _ => return,
    };
```

Now we can switch:

```
fn world_handler(req: Request) -> Box<Hello> {
    Box::new(Hello { greeting: "Hälló, wørld".to_string() })
}

fn mars_handler(req: Request) -> Box<Hello> {
    Box::new(Hello { greeting: "Hälló, márs".to_string() })
}

fn handler(req: Request, res: Response) {
    let path = match req.uri {
        uri::RequestUri::AbsolutePath(ref path) => path.clone(),
        _ => return,
    };

    let hw = match path.as_ref() {
        "/world" => world_handler(req),
        "/mars" => mars_handler(req),
        _ => return, // 404
    };

    // Create a new WriteWrapper for the response
    let wrapped = &mut WriteWrap { res: res.start().unwrap() };

    // Create a new json::Encoder with the wrapped writer
    let enc = &mut json::Encoder::new(wrapped);

    // The [RustcEncodable] trait adds the 'encode' method to the hw struct.
    hw.encode(enc).unwrap();
}
```


That'll do for today.

git tag v0.0.3

Step 4: Data
============

My first attempt at adding data was a global static, this is surprisingly
challenging in rust, probably for the same reason I'm going to skip it and go
right for a real 'database': That's how it will work the end product.

For this booking system, there are 'timeslots', and 'people'. 0 -> 1 people per
timeslot, 1 timeslot per person. It's a pretty easy data structure, effectively
it's a timeslot with an optional person. But what if a person could book twice,
or two people could share a 'group booking', for that future (which, after
writing this 5 or so times, I know isn't coming, at least with this code
version) I will use a relational DB with two tables, timeslot and person.

Also, because I want to shove this in a docker container and forget about it,
and because there are probably, at most, 100 users - I'm going to use sqlite.

Firstly, some imports.
```
[dependencies]
hyper = "*"
rustc-serialize = "*"

#new:
rusqlite = "*"

[dependencies.time]
version = "*"
features = ["rustc-serialize"]
```

We are including the serialize 'feature' of the time package, which allows us
to use it just like our Hello struct. The default serialization isn't what we
will want in the end, but will do for now

```rust
fn times_handler<'a>(req: Request) -> Vec<Timeslot> {

    // TODO: This should, obviously, not be in the handler.
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE timeslot(
     time TIMESTAMP NOT NULL PRIMARY KEY
     )",
                 &[])
        .unwrap();

    let t = Timeslot { time: time::get_time() };
    conn.execute("INSERT INTO timeslot (time) VALUES ($1)", &[&t.time]).unwrap();

    // Begin real handler code.

    // Select all
    let mut stmt = conn.prepare("SELECT time FROM timeslot").unwrap();

    // Turn them into an iterator
    let mut timeslot_iter = stmt.query_map(&[], |row| Timeslot { time: row.get(0) })
                                .unwrap();

    // For now, convert the iterator into a vector, later, maybe we can encode the json on the fly?
    let mut timeslots: Vec<Timeslot> = Vec::new();
    for ts in timeslot_iter {
        timeslots.push(ts.unwrap());
    }
    timeslots
}
```

And just a little bug this brings out in the writer -> writer wrapper. The
json encoder for 'time' seems to call write with 0 bytes, which must write a
null byte or something, because curl stops reading.

As a quick debug, I used telnet to make the http request, and... well, it's
interesting. For now I'll leave this as an exercise for the reader, but I'll be
coming back to it later on.

To 'quickfix' it for now, I just added a length check in the wrapper.

```rust
impl<'a> core::fmt::Write for WriteWrap<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Attempt to write to the response, and unwrap the error to make a fmt::Result instead of
        // an io::Result
        //
        if (s.len() < 1) {
            return Ok(());
        }

        match self.res.write(s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(core::fmt::Error),
        }
    }
}
```


Moving along (nothing to see here) - I'll make the database a static singleton first.
The internet appears to suggest very subtly that singletons and statics make no
sense in rust, and that I might consider 'passing context objects' instead. But
how do I do that? My handlers are functions, somehow I have to give context to
those functions.

The hyper handler function actually takes 'trait', which is odd, because I'm
passing it a function. My guess is that, since the trait only has one required
method, a function which has the same signature without the &self ref must
automatically implement the trait. Either way, to put context in, I'll be using
a struct instead.

```rust
struct handler {
	conn: Mutex<Connection>,
};

impl hyper::server::Handler for handler {
	fn handle(&self, req: Request, res: Response) {
		// as above
        let hw = match path.as_ref() {
            "/times" => times_handler(req, &self.conn.lock().unwrap()),
            _ => return, // 404
        };
	}
}	

fn times_handler<'a>(req: Request, conn: &Connection) -> Vec<Timeslot> {
	// use the database connection
}


fn main() {
	let conn = Connection::open_in_memory().unwrap();
	// set up some data
	// use the conn in a Mutex.
    let handler = Handler { conn: Mutex::new(conn) };
    Server::http("0.0.0.0:8080").unwrap().handle(handler).unwrap();
}
```

This works, but it's terrible code design - The sqlite library only handles one
call at a time (not sure why, I'm fairly sure sqlite can do multiple threads,
but anyway) so I've had to wrap it in a mutex. That's all fine, but the place
it's wrapped is prior to calling the handler (imagine it prior to calling ANY
handler), meaning the server can only deal with one request at a time.

To improve that, I'll create a 'data' struct which will wrap up all database
calls, and can lock the mutex only when required. Already I'm scared about how
the whole borrowing thing will work here - Will I be able to return the
query_map iterator? If so, when will the mutex be released? (The latter is more
about how the library works than how rust does borrowing)

But before all of that - this code is a MESS! One file for the whole thing?

git tag 0.0.4

Part 5: Organising Code
=======================

The first cleanup: Our handler method returns a concrete type. That's OK
because we only have one type, but won't cut it later. It looks like returning,
say, 'something which implements the Encodable trait' is in the too hard
basket, and not really how things are done - so will tackle that challenge
later. So I've refactored a bit to make part after the path matcher into a
function that the handler functions themselves can call. This prevents the
issue above, but also provides more control to how the response is written.

Secondly - that whole write wrapper and method and such are very much 'their
own thing', so I've made them into a module.

