use lazy_regex::bytes_regex_captures;

pub enum GameEvent {
    MapChange { code: Vec<u8>, name: Vec<u8> },
    MultiplayerStarted(Vec<u8>),
}

pub fn log_line_parser(line: &[u8]) -> Option<GameEvent> {
    // MM1CUT - CUT MAN
    if let Some((_, code, name)) = bytes_regex_captures!(r"^([A-Z0-9]{1,8})\s*-\s*(.+)$", line) {
        return Some(GameEvent::MapChange {
            code: code.to_vec(),
            name: name.to_vec(),
        });
    }

    // Connecting to 127.0.0.1:10666
    if let Some((_, address)) = bytes_regex_captures!(r"^Connecting to (\S+:\d+)$", line) {
        return Some(GameEvent::MultiplayerStarted(address.to_vec()));
    }

    None
}
