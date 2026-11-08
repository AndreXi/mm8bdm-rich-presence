use std::io::Error as IOError;

use huffman_zandronum::{DecodeError, EncodeError};

#[derive(Debug)]
pub enum ServerRequestError {
    CannotBindSocket,
    CodecBufferLimitReached,

    IgnoredRequestError,
    BannedRequestError,
    UnknownRequestError,
}

impl From<IOError> for ServerRequestError {
    fn from(_: IOError) -> Self {
        Self::CannotBindSocket
    }
}

impl From<DecodeError> for ServerRequestError {
    fn from(_: DecodeError) -> Self {
        Self::CodecBufferLimitReached
    }
}

impl From<EncodeError> for ServerRequestError {
    fn from(_: EncodeError) -> Self {
        Self::CodecBufferLimitReached
    }
}
