pub(crate) const MAX_NETWORK_BUFFER_SIZE: usize = 1 << 10;
pub(crate) const MAX_CODEC_OUTPUT_SIZE: usize = 1 << 10;

pub(super) const LAUNCHER_CHALLENGE: u32 = 199;

pub(super) const SQF_GAMETYPE: u32 = 0x00000080;
pub(super) const SQF_NUMPLAYERS: u32 = 0x00080000;
pub(super) const SQF_MAXPLAYERS: u32 = 0x00000020;
pub(super) const SQF_MAPNAME: u32 = 0x00000008;

pub(super) const SERVER_LAUNCHER_CHALLENGE: u32 = 5660023;
pub(super) const SERVER_LAUNCHER_IGNORING: u32 = 5660024;
pub(super) const SERVER_LAUNCHER_BANNED: u32 = 5660025;
