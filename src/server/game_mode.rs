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

pub struct GameModeData {
    pub mode: GameMode,
    pub instagib: bool,
    pub buckshot: bool,
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
    pub(crate) fn as_str(&self) -> &'static str {
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
