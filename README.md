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
