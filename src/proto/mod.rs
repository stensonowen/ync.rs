use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;
use tokio_proto::pipeline::ServerProto;
use std::io;

pub mod service;
pub mod codec;
use self::codec::*;

pub struct LineProto;


pub struct Request(String); 


impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for LineProto {
    /// For this protocol style, `Request` matches the `Item` type of the codec's `Encoder`
    type Request = Request;

    /// For this protocol style, `Response` matches the `Item` type of the codec's `Decoder`
    type Response = Request;

    /// A bit of boilerplate to hook in the codec:
    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}

