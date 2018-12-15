//!
//! The example of using raw futures on a toy example.
//!

#![feature(async_await, await_macro, futures_api, pin, arbitrary_self_types)]

use futures::executor::ThreadPool;
use futures::prelude::*;
use futures::task::LocalWaker;
use futures::Poll;
use futures::Stream;
use std::cell::Cell;
use std::pin::Pin;
use std::thread;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug)]
enum MyTask {
    // Sleeps specified time interval.
    Sleep { start: Instant, duration: Duration },
    // Loops specified number of time. Imitates cpu-intensive work.
    Task { id: usize, loops: Cell<u64> },
    // Writes the message to stdout
    Log { msg: String },
}

unsafe impl Send for MyTask {}

impl MyTask {
    fn sleep(duration: Duration) -> Self {
        MyTask::Sleep {
            start: Instant::now(),
            duration,
        }
    }
    fn task(id: usize, loops: u64) -> Self {
        MyTask::Task {
            id,
            loops: Cell::new(loops),
        }
    }
    fn log(msg: &'static str) -> Self {
        MyTask::Log {
            msg: msg.to_string(),
        }
    }
}

impl Future for MyTask {
    type Output = Result<(), ()>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<<Self as Future>::Output> {
        match *self {
            MyTask::Sleep { start, duration } => {
                if Instant::now().duration_since(start) >= duration {
                    Poll::Ready(Ok(()))
                } else {
                    lw.wake();
                    Poll::Pending
                }
            }
            MyTask::Log { ref msg } => {
                println!("{}", msg);
                Poll::Ready(Ok(()))
            }
            MyTask::Task { id, ref loops } => {
                /* from tokio docs (https://tokio.rs/docs/futures/basic/)
                  When a future’s poll function is called, the implementation will synchronously
                  do as much work as possible until it is logically blocked on some asynchronous
                  event that has not occured yet. The future implementation then saves its state
                  internally so that the next time poll is called (after an external event is
                  receied), it resumes processing from the point it left off. Work is not repeated.
                */

                loop {
                    // end condition
                    if loops.get() == 0 {
                        return Poll::Ready(Ok(()));
                    }

                    // save current state
                    loops.set(loops.get() - 1);

                    // do some work
                    thread::sleep(Duration::from_millis(1));

                    // break off, return control temporarily
                    if loops.get() % 1000 == 0 {
                        println!("[{}] Loops left {}", id, loops.get());
                        lw.wake();
                        return Poll::Pending;
                    }
                }
            }
        }
    }
}

fn main() {
    println!("Future example starts!");

    // todo write macros for ease combinations

    let future1 = MyTask::log("[10] Doing work...")
        .and_then(|_| MyTask::task(10, 2500))
        .and_then(|_| MyTask::log("[11] Wait 3 sec..."))
        .and_then(|_| MyTask::sleep(Duration::from_secs(3)))
        .and_then(|_| MyTask::log("[12] Wait 1 sec..."))
        .and_then(|_| MyTask::sleep(Duration::from_secs(1)))
        .and_then(|_| MyTask::log("[13] Doing work..."))
        .and_then(|_| MyTask::task(10, 2500))
        .map_err(|err| println!("{:?}", err))
        .map(|_| ());

    let future2 = MyTask::log("[20] Doing work...")
        .and_then(|_| MyTask::task(20, 4500))
        .and_then(|_| MyTask::log("[21] Doing work..."))
        .and_then(|_| MyTask::task(21, 1500))
        .and_then(|_| MyTask::log("[22] Doing work..."))
        .and_then(|_| MyTask::task(21, 3500))
        .map_err(|err| println!("{:?}", err))
        .map(|_| ());

    let start = MyTask::log("My Async app is started");
    let end = MyTask::log("That is the end.");

    // compute fibonacci with help of streams

    let fib = fibonacci().take(90).for_each(|res| {
        MyTask::Log {
            msg: format!("fibonacci is {:?}", res),
        }
        .map(|_| ())
    });

    let app = start
        // future1 and future2 will be performed parallel
        .and_then(|_| fib.map(|_| Ok(())))
        .and_then(|_| future1.join(future2).map(|_| Ok(())))
        .and_then(|_| end);

    ThreadPool::new()
        .expect("Failed to create threadpool")
        .run(app)
        .unwrap();
}

// Illustrates how to pass and return Future
#[allow(dead_code)]
fn add_10<F>(f: F) -> impl Future<Output = i32>
where
    F: Future<Output = i32>,
{
    f.map(|i| i + 10)
}

fn fibonacci() -> impl Stream<Item = u64> {
    stream::unfold((1, 1), |(curr, next)| {
        let new_next = curr + next;
        future::ready(Some((curr, (next, new_next))))
    })
}
