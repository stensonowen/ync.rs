extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_service;
extern crate tokio_proto;
extern crate bytes;
extern crate tempdir;
extern crate base64;
extern crate md5;

use std::net::{Ipv4Addr,IpAddr,SocketAddr};
use std::env;

mod proto;
use proto::LineProto;
use proto::service::Rsync;
use tokio_proto::TcpServer;

//mod client;

const USAGE: &'static str = "USAGE: ./syncr client|server <port>";


fn main() {
    let mut args = env::args();
    let mode = args.nth(1).expect(USAGE);
    let port: u16 = args.nth(0).expect(USAGE).parse().expect(USAGE);
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

    if mode == "server" {
        let server = TcpServer::new(LineProto, addr);
        server.serve(|| Rsync::new());
    } else if mode == "client" {
        panic!("Client unimplemented; use netcat at base64 encode by hand for now");
        //client::client(addr);

    } else {
        panic!("Invalid mode string; it must be `client` or `server`");
    }

}
