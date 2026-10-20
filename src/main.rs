use bstr::{ByteSlice, io::BufReadExt};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use lazy_regex::bytes_regex_captures;
use std::io::{self};

const CLIENT_ID: &str = "1561172546756280430";

fn capture_map_name(line: &[u8]) -> Option<(&[u8], &[u8])> {
    let (_, code, name) = bytes_regex_captures!(r"^([A-Z0-9]{1,8})\s*-\s*(.+)$", line)?;
    Some((code, name))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DiscordIpcClient::new(CLIENT_ID);
    match client.connect() {
        Ok(_) => println!("[RPC]: Connected!"),
        Err(e) => {
            eprintln!("[RPC]: {e:?} - {e:#}");
        }
    }

    let stdin = io::stdin();
    let reader = stdin.lock();

    for line_result in reader.byte_lines() {
        let line = line_result?;

        if let Some((code, name)) = capture_map_name(&line) {
            let payload = activity::Activity::new()
                .state(format!("{} - {}", code.as_bstr(), name.as_bstr()))
                .details("Singleplayer");
            let _ = client.set_activity(payload);
        }

        println!("{}", line.as_bstr());
    }

    Ok(())
}
