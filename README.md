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
