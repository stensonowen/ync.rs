use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;
use tokio_proto::pipeline::ServerProto;
use std::io;

const SRV_DIRECTORY: &'static str = "rsync_dir";
const MANIFEST_FILE: &'static str = ".4220_file_list.txt";

pub mod line;
use self::line::{LineIn,LineOut};
pub mod service;
pub mod codec;
use self::codec::*;

pub struct LineProto;


impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for LineProto {
    type Request = LineIn;
    type Response = LineOut;

    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}

