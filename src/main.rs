use bstr::{ByteSlice, io::BufReadExt};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use std::io::{self};

const CLIENT_ID: &str = "1561172546756280430";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*
     * Discord RPC connection
     */
    let mut client = DiscordIpcClient::new(CLIENT_ID);
    match client.connect() {
        Ok(_) => println!("[RPC]: Connected!"),
        Err(e) => {
            eprintln!("[RPC]: {e:?} - {e:#}");
        }
    }

    let payload = activity::Activity::new().details("Singleplayer");
    let _ = client.set_activity(payload);

    /*
     * Zandronum logs capture
     */
    let stdin = io::stdin();
    let reader = stdin.lock();

    for line_result in reader.byte_lines() {
        let line = line_result?;
        println!("{}", line.as_bstr());
    }

    Ok(())
}
