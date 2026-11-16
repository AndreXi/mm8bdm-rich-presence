use std::io::{Cursor, Result as IOResult};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::server::{
    constants::{
        SERVER_LAUNCHER_BANNED, SERVER_LAUNCHER_CHALLENGE, SERVER_LAUNCHER_IGNORING, SQF_GAMETYPE,
        SQF_MAPNAME, SQF_MAXPLAYERS, SQF_NUMPLAYERS,
    },
    error::ServerRequestError,
    game_mode::{GameMode, GameModeData},
    read_zstr::ReadZStr,
};

pub struct ServerResponseData {
    pub map_name: Option<Vec<u8>>,
    pub max_players: Option<u8>,
    pub game_mode: Option<GameModeData>,
    pub num_players: Option<u8>,
}

pub(super) struct ServerResponse<'a> {
    _time: u32,
    _version: Vec<u8>,
    flags: u32,
    buffer: Cursor<&'a [u8]>,
    parsed: bool,
}

impl<'a> ServerResponse<'a> {
    fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag != 0
    }

    fn read_map_name(&mut self) -> IOResult<Option<Vec<u8>>> {
        if !self.has_flag(SQF_MAPNAME) {
            Ok(None)
        } else {
            self.buffer.read_zstr().map(Some)
        }
    }

    fn read_max_players(&mut self) -> IOResult<Option<u8>> {
        if !self.has_flag(SQF_MAXPLAYERS) {
            Ok(None)
        } else {
            self.buffer.read_u8().map(Some)
        }
    }

    fn read_num_players(&mut self) -> IOResult<Option<u8>> {
        if !self.has_flag(SQF_NUMPLAYERS) {
            Ok(None)
        } else {
            self.buffer.read_u8().map(Some)
        }
    }

    fn read_game_mode(&mut self) -> IOResult<Option<GameModeData>> {
        if !self.has_flag(SQF_GAMETYPE) {
            Ok(None)
        } else {
            let data: GameModeData = GameModeData {
                mode: GameMode::from(self.buffer.read_u8()?),
                instagib: self.buffer.read_u8()? != 0,
                buckshot: self.buffer.read_u8()? != 0,
            };

            Ok(Some(data))
        }
    }

    pub(super) fn new(bytes: &'a [u8]) -> Result<Self, ServerRequestError> {
        let mut buffer = Cursor::new(bytes);

        let challenge = buffer.read_u32::<LittleEndian>()?;
        match challenge {
            SERVER_LAUNCHER_IGNORING => return Err(ServerRequestError::IgnoredRequestError),
            SERVER_LAUNCHER_BANNED => return Err(ServerRequestError::BannedRequestError),
            SERVER_LAUNCHER_CHALLENGE => {}
            _ => return Err(ServerRequestError::UnknownRequestError),
        }

        let time = buffer.read_u32::<LittleEndian>()?;
        let version = buffer.read_zstr()?;
        let flags = buffer.read_u32::<LittleEndian>()?;

        Ok(Self {
            _time: time,
            _version: version,
            flags,
            buffer,
            parsed: false,
        })
    }

    pub(super) fn parse(&mut self) -> Result<Option<ServerResponseData>, ServerRequestError> {
        if self.parsed {
            return Ok(None);
        }
        self.parsed = true;

        let map_full_name = self.read_map_name()?;
        let max_players = self.read_max_players()?;
        let game_mode = self.read_game_mode()?;
        let num_players = self.read_num_players()?;

        Ok(Some(ServerResponseData {
            map_name: map_full_name,
            max_players,
            game_mode,
            num_players,
        }))
    }
}
