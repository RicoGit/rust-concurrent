//!
//! The example of using sdtlib concurrency primitives on a toy example.
//!
//! Used: Send, Thread, Channel, AtomicUsize
//!

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

enum MyTask {
    Shutdown,
    Job(u64),
}

unsafe impl Send for MyTask {}

fn main() {
    // create channel
    let (sender, receiver) = channel();

    println!("Push one task");
    sender.send(MyTask::Job(3)).unwrap();

    thread::sleep(Duration::from_secs(1));

    println!("Push another task");
    sender.send(MyTask::Job(1)).unwrap();

    thread::sleep(Duration::from_secs(1));

    println!("Push shutdown");
    sender.send(MyTask::Shutdown).unwrap();

    start_executor_service(receiver);
}

fn start_executor_service(receiver: Receiver<MyTask>) {
    println!("Start event loop");

    let counter = AtomicUsize::new(0);

    let event_loop = thread::spawn(move || {
        loop {
            println!("Waiting for a new tasks.");
            let task = receiver.recv();

            counter.fetch_add(1, Ordering::SeqCst);

            match task {
                Ok(MyTask::Job(load)) => {
                    println!("Job({}) is received and processing...", load);
                    thread::sleep(Duration::from_secs(load));
                }
                Ok(MyTask::Shutdown) => {
                    println!("Shutdown is received.");
                    break;
                }
                Err(err) => {
                    println!("Error is received {:?}", err);
                }
            }
        }

        counter
    });

    println!("Stop event loop, executed {:?}", event_loop.join().unwrap());
}
