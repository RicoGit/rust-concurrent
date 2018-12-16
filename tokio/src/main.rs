//!
//! Udp server that receives commands and do executes theirs.
//!

#[macro_use]
extern crate futures;

use futures::prelude::*;
use std::env;
use std::net::SocketAddr;
use tokio::io;
use tokio::net::UdpSocket;

// todo use TCP, establish connection, read cmd, deserialize and execute
fn main() -> Result<(), Box<std::error::Error>> {
    println!("Server starts with {:?}", env::args());

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>()?;

    let socket = UdpSocket::bind(&addr)?;
    println!("Listening on: {}", socket.local_addr()?);

    let server = Server {
        socket,
        buf: vec![0; 1024],
        to_send: None,
    };

    // `tokio::run` spawns the task on the Tokio runtime and starts running.
    tokio::run(server.map_err(|e| println!("server error = {:?}", e)));
    Ok(())
}

struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    to_send: Option<(usize, SocketAddr)>,
}

impl Future for Server {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            // First we check to see if there's a message we need to echo back.
            // If so then we try to send it back to the original source, waiting
            // until it's writable and we're able to do so.
            if let Some((size, peer)) = self.to_send {
                let amt = try_ready!(self.socket.poll_send_to(&self.buf[..size], &peer));
                println!("Echoed {}/{} bytes to {}", amt, size, peer);
                self.to_send = None;
            }

            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            self.to_send = Some(try_ready!(self.socket.poll_recv_from(&mut self.buf)));
        }
    }
}
