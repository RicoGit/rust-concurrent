//!
//! The example of using std::future with async/await on a toy example.
//!
//! for additional information see the article: https://jsdw.me/posts/rust-asyncawait-preview/
//! and examples: https://github.com/jsdw/jsdw.me/blob/master/content/posts/rust-asyncawait-preview/src/main.rs

// enable the await! macro, async support, and the new std::Futures api.
#![feature(await_macro, async_await, futures_api, gen_future)]
// only needed if we want to manually write a method to go forward from 0.1 to 0.3 future,
// or manually implement a std future (it provides Pin and Unpin):
#![feature(pin)]
// only needed to manually implement a std future:
#![feature(arbitrary_self_types)]

#[macro_use]
extern crate tokio;
use tokio::prelude::*;

use futures::Future as OldFuture;
use std::future::Future as NewFuture;

async fn sleep(n: u64) {
    use std::time::{Duration, Instant};
    use tokio::timer::Delay;
    await!(Delay::new(Instant::now() + Duration::from_secs(n))).unwrap();
}

// read line from std, capitalize and write to stdout
fn main() {
    println!("Future with async/await starts!");

    // transmission through the channel for this case is a bad approach, it is
    // just a learning example. Stream should be better here
    let (mut cons, prod) = futures::sync::mpsc::channel::<Vec<u8>>(8);

    let stdin_reader = async move {
        let mut buf = [0; 16];
        let mut stdin = tokio::io::stdin();

        // While read_async returns a number of bytes
        while let Ok(n) = await!(stdin.read_async(&mut buf)) {
            // bail if we've read everything:
            if n == 0 {
                break;
            };
            // print each byte we read up to a newline/error:
            println!("Produced: {}", String::from_utf8_lossy(&buf[..n]));
            cons.try_send(Vec::from(&buf[..n])).unwrap();
        }
    };

    // blocks thread until all futures will complete
    tokio::run_async(
        async move {
            // reads bytes from stdin and puts their to channel
            tokio::spawn_async(stdin_reader);

            // todo add blocking stage (working with network for example)

            // reads channel and puts all incoming bytes to stdout
            tokio::spawn(
                prod.for_each(|bytes| {
                    println!("Consumed: {}", String::from_utf8(bytes).unwrap());
                    future::ok(())
                })
                .map(|_| ()),
            );

            // await async function
            await!(sleep(1));
            println!("after await");
            // await old future
            await!(future::lazy(|| {
                println!("Hello from future");
                future::ok::<(), ()>(())
            })).unwrap();
            println!("Await async variable {:?}", await!(async { "Hello World" }));
        },
    );
}

// converts from a new style Future to an old style one:
#[allow(dead_code)]
fn backward<I, E>(f: impl NewFuture<Output = Result<I, E>>) -> impl OldFuture<Item = I, Error = E> {
    tokio_async_await::compat::backward::Compat::new(f)
}
