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


