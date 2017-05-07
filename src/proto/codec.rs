use std::{io, str};

use tokio_io::codec::{Decoder, Encoder};
use bytes::BytesMut;

pub struct LineCodec;
use super::line::{LineIn,LineOut};

impl Decoder for LineCodec {
    type Item = LineIn;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<LineIn>> {
        if let Some(i) = buf.iter().position(|&b| b == b'\n') {
            // remove the serialized frame from the buffer.
            let line = buf.split_to(i);

            // Also remove the '\n'
            buf.split_to(1);

            // Turn this data into a UTF string and return it in a Frame.
            match str::from_utf8(&line) {
                Ok(s) => Ok(Some(LineIn::parse(s))),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "bad utf8")),
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder for LineCodec {
    type Item = LineOut;
    type Error = io::Error;

    fn encode(&mut self, msg: LineOut, buf: &mut BytesMut) -> io::Result<()> {
        buf.extend(msg.as_bytes());
        buf.extend(b"\n");
        Ok(())
    }
}

