extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_service;
extern crate tokio_proto;
extern crate bytes;
extern crate tempdir;
extern crate base64;
extern crate md5;


mod proto;
use proto::LineProto;
use proto::service::Rsync;
use tokio_proto::TcpServer;

fn main() {
    // Specify the localhost address
    let addr = "0.0.0.0:12345".parse().unwrap();

    // The builder requires a protocol and an address
    let server = TcpServer::new(LineProto, addr);

    server.serve(|| Rsync::new());
}
