use bstr::ByteSlice;
use lazy_regex::bytes_regex_captures;

pub enum GameEvent {
    MapChange { code: Vec<u8> },
    MultiplayerStarted(Vec<u8>),
    MultiplayerConnected,
}

pub fn log_line_parser(line: &[u8]) -> Option<GameEvent> {
    // MM1CUT - CUT MAN
    if let Some((_, code, _name)) = bytes_regex_captures!(r"^([A-Z0-9]{1,8})\s*-\s*(.+)$", line) {
        println!("MATCH! map {:?}", code.as_bstr());
        return Some(GameEvent::MapChange {
            code: code.to_vec(),
        });
    }

    // Connecting to 127.0.0.1:10666
    if let Some((_, address)) = bytes_regex_captures!(r"^Connecting to (\S+:\d+)$", line) {
        println!("MATCH! address {:?}", address.as_bstr());
        return Some(GameEvent::MultiplayerStarted(address.to_vec()));
    }

    // Connected!
    if let Some(_) = bytes_regex_captures!(r"^Connected!$", line) {
        return Some(GameEvent::MultiplayerConnected);
    }

    None
}
