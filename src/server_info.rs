use bstr::ByteSlice;
use byteorder::{LittleEndian, ReadBytesExt};
use huffman_zandronum::{decode, encode};
use std::io::{Cursor, Error};
use std::net::UdpSocket;
use std::result::Result;
use std::time::Duration;

use crate::read_zstr::ReadZStr;

const LAUNCHER_CHALLENGE: i32 = 199;
const MAX_CODEC_OUTPUT_SIZE: usize = 256;

const SQF_GAMETYPE: u32 = 0x00000080;
const SQF_NUMPLAYERS: u32 = 0x00080000;
const SQF_MAXPLAYERS: u32 = 0x00000020;

const SERVER_LAUNCHER_CHALLENGE: u32 = 5660023;
const SERVER_LAUNCHER_IGNORING: u32 = 5660024;

pub enum GameMode {
    Cooperative = 0,
    Survival = 1,
    Invasion = 2,
    Deathmatch = 3,
    Teamplay = 4,
    Duel = 5,
    Terminator = 6,
    LastManStanding = 7,
    TeamLms = 8,
    Possession = 9,
    TeamPossession = 10,
    TeamGame = 11,
    Ctf = 12,
    OneFlagCtf = 13,
    Skulltag = 14,
    Domination = 15,
    Unknown = 0xFF,
}

impl From<u8> for GameMode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Cooperative,
            1 => Self::Survival,
            2 => Self::Invasion,
            3 => Self::Deathmatch,
            4 => Self::Teamplay,
            5 => Self::Duel,
            6 => Self::Terminator,
            7 => Self::LastManStanding,
            8 => Self::TeamLms,
            9 => Self::Possession,
            10 => Self::TeamPossession,
            11 => Self::TeamGame,
            12 => Self::Ctf,
            13 => Self::OneFlagCtf,
            14 => Self::Skulltag,
            15 => Self::Domination,
            _ => Self::Unknown,
        }
    }
}

impl GameMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cooperative => "Cooperative",
            Self::Survival => "Survival",
            Self::Invasion => "Invasion",
            Self::Deathmatch => "Deathmatch",
            Self::Teamplay => "Teamplay",
            Self::Duel => "Duel",
            Self::Terminator => "Terminator",
            Self::LastManStanding => "Last Man Standing",
            Self::TeamLms => "Team LMS",
            Self::Possession => "Possession",
            Self::TeamPossession => "Team Possession",
            Self::TeamGame => "Team Game",
            Self::Ctf => "Capture the Flag",
            Self::OneFlagCtf => "One-Flag CTF",
            Self::Skulltag => "Skulltag",
            Self::Domination => "Domination",
            Self::Unknown => "Unknown",
        }
    }
}

pub struct ServerInfo {
    pub players: u8,
    pub max_players: u8,
    pub game_mode: GameMode,
    pub instagib: bool,
    pub buckshot: bool,
}

pub enum ServerInfoError {
    IgnoredRequestError,
    RejectedRequestError,
    IoError(Error),
}

impl From<Error> for ServerInfoError {
    fn from(value: Error) -> Self {
        Self::IoError(value)
    }
}

pub fn get_server_info(server_address: &str) -> Result<ServerInfo, ServerInfoError> {
    let mut codec_output = Vec::with_capacity(MAX_CODEC_OUTPUT_SIZE);
    let start_time: u32 = 0;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(3)))?;

    let mut packet = Vec::new();
    packet.extend_from_slice(&LAUNCHER_CHALLENGE.to_le_bytes());

    let request_flags: u32 = SQF_GAMETYPE | SQF_NUMPLAYERS | SQF_MAXPLAYERS;
    packet.extend_from_slice(&request_flags.to_le_bytes());
    packet.extend_from_slice(&start_time.to_le_bytes());
    encode(&packet, &mut codec_output, MAX_CODEC_OUTPUT_SIZE).expect("TODO: Handle later!");
    println!("SEND: {:02x?} | encoded: {:02x?}", packet, codec_output);

    socket.send_to(&codec_output, server_address)?;
    println!("Query sent to {}, waiting for response...", server_address);

    let mut buffer = [0u8; MAX_CODEC_OUTPUT_SIZE];
    let (amt, _) = socket.recv_from(&mut buffer)?;

    let received_len = amt;
    decode(&buffer[..amt], &mut codec_output, MAX_CODEC_OUTPUT_SIZE).expect("TODO: Handle later");
    println!("RECV: {:02x?} | decoded: {:02x?}", buffer, codec_output);

    let decoded = &codec_output;
    let decoded_len = decoded.len();

    println!(
        "Received: {received_len} bytes, decoded: {decoded_len} bytes ({:.1}% of received size)",
        if received_len == 0 {
            0.0
        } else {
            decoded_len as f64 / received_len as f64 * 100.0
        }
    );

    // Parse the server first packet
    let mut cursor = Cursor::new(decoded);

    // First 4 bytes match the sent time/challenge identifier
    let response_challenge = cursor.read_u32::<LittleEndian>()?;

    // Handle ignore responses
    if response_challenge == SERVER_LAUNCHER_IGNORING {
        return Err(ServerInfoError::IgnoredRequestError);
    }

    // Handle banned or other
    if response_challenge != SERVER_LAUNCHER_CHALLENGE {
        return Err(ServerInfoError::RejectedRequestError);
    }

    let response_time = cursor.read_u32::<LittleEndian>()?;
    let version = cursor.read_zstr()?;
    let flags = cursor.read_u32::<LittleEndian>()?;

    let max_players = cursor.read_u8()?;

    // Read 3 bytes of SQF_GAMETYPE
    let game_type = GameMode::from(cursor.read_u8()?);
    let instagib: bool = cursor.read_u8()? == 1;
    let buckshot: bool = cursor.read_u8()? == 1;

    let num_players = cursor.read_u8()?;

    println!("--- Server Info (Challenge: {}) ---", response_challenge);
    println!("--- Server Info (s: {}) ---", response_time);
    println!("--- Server Info (ver: {}) ---", version.as_bstr());
    println!("--- Server Info (flags: {}) ---", flags);
    println!(
        "--- Server Info (SQF_GAMETYPE: {}) t:{} i:{} b:{} ---",
        flags & SQF_GAMETYPE == SQF_GAMETYPE,
        game_type.as_str(),
        instagib,
        buckshot
    );
    println!(
        "--- Server Info (SQF_NUMPLAYERS: {}) {} ---",
        flags & SQF_NUMPLAYERS == SQF_NUMPLAYERS,
        num_players
    );
    println!(
        "--- Server Info (SQF_MAXPLAYERS: {}) {} ---",
        flags & SQF_MAXPLAYERS == SQF_MAXPLAYERS,
        max_players
    );

    println!(
        "--- Server Info (SQF_PLAYERDATA: {}) ---",
        flags & 0x00100000 == 0x00100000
    );

    Ok(ServerInfo {
        players: num_players,
        max_players,
        game_mode: game_type,
        instagib: instagib,
        buckshot: buckshot,
    })
}
