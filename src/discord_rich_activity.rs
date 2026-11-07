use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use std::error::Error;

use crate::server_info::ServerInfo;

const JOIN_URL_HOST: &str = "https://trillster.dev/";

pub struct DiscordRichActivityClient {
    client: DiscordIpcClient,
}

impl DiscordRichActivityClient {
    pub fn connect(client_id: &str) -> Result<Self, Box<dyn Error>> {
        let mut client = DiscordIpcClient::new(client_id);
        client.connect()?;

        Ok(Self { client })
    }

    pub fn update_singleplayer(&mut self, map: &str) -> Result<(), Box<dyn Error>> {
        let payload = activity::Activity::new().details("Singleplayer").state(map);

        self.client.set_activity(payload)?;
        Ok(())
    }

    pub fn update_multiplayer(
        &mut self,
        map: &str,
        server_address: &str,
        server_info: &ServerInfo,
    ) -> Result<(), Box<dyn Error>> {
        let details = format!(
            "{} - {}/{} players",
            server_info.game_mode.as_str(),
            server_info.players,
            server_info.max_players,
        );

        let mut payload = activity::Activity::new()
            .details(details)
            .state(map)
            .activity_type(activity::ActivityType::Playing);

        if let Some((ip, port)) = parse_server_address(server_address) {
            let join_url = format!(
                "{JOIN_URL_HOST}?project=JoinServer&ip={}&port={port}&source=drpc",
                encode_query_component(ip),
            );
            payload = payload.buttons(vec![activity::Button::new("Join Server", join_url)]);
        }

        self.client.set_activity(payload)?;
        Ok(())
    }
}

fn parse_server_address(address: &str) -> Option<(&str, u16)> {
    let (ip, port) = address.rsplit_once(':')?;
    let ip = ip
        .strip_prefix('[')
        .and_then(|ip| ip.strip_suffix(']'))
        .unwrap_or(ip);
    let port = port.parse().ok()?;

    if ip.is_empty() {
        return None;
    }

    Some((ip, port))
}

fn encode_query_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{encode_query_component, parse_server_address};

    #[test]
    fn parses_ipv4_server_address() {
        assert_eq!(
            parse_server_address("127.0.0.1:10666"),
            Some(("127.0.0.1", 10666))
        );
    }

    #[test]
    fn parses_bracketed_ipv6_server_address() {
        assert_eq!(
            parse_server_address("[2001:db8::1]:10666"),
            Some(("2001:db8::1", 10666))
        );
    }

    #[test]
    fn encodes_query_components() {
        assert_eq!(encode_query_component("2001:db8::1"), "2001%3Adb8%3A%3A1");
    }
}
