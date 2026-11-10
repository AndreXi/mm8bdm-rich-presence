use std::{net::UdpSocket, time::Duration};

use huffman_zandronum::{decode, encode};

use crate::server::constants::{
    LAUNCHER_CHALLENGE, MAX_CODEC_OUTPUT_SIZE, SQF_GAMETYPE, SQF_MAPNAME, SQF_MAXPLAYERS,
    SQF_NUMPLAYERS,
};
use crate::server::error::ServerRequestError;
use crate::server::server_response::{ServerResponse, ServerResponseData};

const LAUNCHER_PART: [u8; 4] = LAUNCHER_CHALLENGE.to_le_bytes();
const FLAG_PART: [u8; 4] =
    (SQF_MAPNAME | SQF_MAXPLAYERS | SQF_GAMETYPE | SQF_NUMPLAYERS).to_le_bytes();
const TIME_PART: [u8; 4] = 0u32.to_le_bytes();
const INFO_PACKET: &[u8] = [LAUNCHER_PART, FLAG_PART, TIME_PART].as_flattened();

fn request_server<'a>(
    address: &str,
    network_buffer: &'a mut [u8],
    codec_buffer: &'a mut Vec<u8>,
) -> Result<&'a [u8], ServerRequestError> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(3)))?;

    let len_written = encode(INFO_PACKET, codec_buffer, MAX_CODEC_OUTPUT_SIZE)?;
    socket.send_to(&codec_buffer[0..len_written], &address)?;

    let (amt, _) = socket.recv_from(network_buffer)?;
    let len_written = decode(&network_buffer[0..amt], codec_buffer, MAX_CODEC_OUTPUT_SIZE)?;

    Ok(&codec_buffer[0..len_written])
}

pub(crate) fn fetch_server_info<'a>(
    address: &str,
    network_buffer: &'a mut [u8],
    codec_buffer: &'a mut Vec<u8>,
) -> Result<Option<ServerResponseData>, ServerRequestError> {
    let response_bytes = request_server(address, network_buffer, codec_buffer)?;
    let mut response = ServerResponse::new(response_bytes)?;
    let data = response.parse()?;

    Ok(data)
}
