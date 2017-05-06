extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_service;
extern crate tokio_proto;
extern crate bytes;
extern crate tempdir;
extern crate md5;

use std::io;
use futures::{Future, Stream, Sink};
use tokio_io::AsyncRead;
use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;
use tokio_service::{Service, NewService};

mod proto;
use proto::codec::LineCodec;
use proto::LineProto;
use proto::service::Echo;

use tokio_proto::TcpServer;

fn main() {
    // Specify the localhost address
    let addr = "0.0.0.0:12345".parse().unwrap();

    // The builder requires a protocol and an address
    let server = TcpServer::new(LineProto, addr);

    // We provide a way to *instantiate* the service for each new
    // connection; here, we just immediately return a new instance.
    server.serve(|| Ok(Echo));
}
