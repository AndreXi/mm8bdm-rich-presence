mod discord_rich_activity;
mod log_line_parser;
mod log_reader;
mod map_name;
mod read_zstr;
mod server_info;
mod server_requester;

use bstr::ByteSlice;
use std::{
    error::Error,
    sync::mpsc::{self, Receiver, SyncSender, TrySendError},
};

use crate::{
    discord_rich_activity::DiscordRichActivityClient,
    log_line_parser::{GameEvent, log_line_parser},
    log_reader::spawn_log_reader,
    map_name::build_map_name,
    server_info::{ServerInfo, ServerInfoError},
    server_requester::spawn_server_requester,
};

const CLIENT_ID: &str = "1561172546756280430";

pub enum AppEvent {
    LogLine(Vec<u8>),
    LogEnded,
    ServerInfoUpdated {
        address: String,
        result: Result<ServerInfo, ServerInfoError>,
    },
}

enum Session {
    Singleplayer,
    Multiplayer {
        address: Vec<u8>,
        map: Option<String>,
    },
}

struct App {
    session: Session,
    discord: Option<DiscordRichActivityClient>,
    server_queries: SyncSender<String>,
}

impl App {
    fn new(server_queries: SyncSender<String>) -> Self {
        Self {
            session: Session::Singleplayer,
            discord: None,
            server_queries,
        }
    }

    fn handle_event(&mut self, event: AppEvent) -> bool {
        match event {
            AppEvent::LogLine(line) => self.handle_log_line(&line),
            AppEvent::ServerInfoUpdated { address, result } => {
                self.handle_server_info(address, result);
            }
            AppEvent::LogEnded => return false,
        }

        true
    }

    fn handle_log_line(&mut self, line: &[u8]) {
        if let Some(event) = log_line_parser(line) {
            self.handle_game_event(event);
        }

        println!("HI! -> {}", line.as_bstr());
    }

    fn handle_game_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::MapChange { code, name } => self.handle_map_change(code, name),
            GameEvent::MultiplayerStarted(address) => {
                self.session = Session::Multiplayer { address, map: None };
            }
        }
    }

    fn handle_map_change(&mut self, code: Vec<u8>, name: Vec<u8>) {
        let map = build_map_name(&code, &name);
        self.connect_discord();

        let server_address = match &mut self.session {
            Session::Singleplayer => None,
            Session::Multiplayer {
                address,
                map: current_map,
            } => {
                *current_map = Some(map.clone());
                Some(address.clone())
            }
        };

        if let Some(address) = server_address {
            self.request_server_info(&address);
        } else {
            self.update_singleplayer(&map);
        }
    }

    fn handle_server_info(&mut self, address: String, result: Result<ServerInfo, ServerInfoError>) {
        let map = match &self.session {
            Session::Multiplayer {
                address: current_address,
                map: Some(map),
            } if current_address.to_str().ok() == Some(address.as_str()) => map.clone(),
            _ => return,
        };

        match result {
            Ok(info) => {
                self.connect_discord();
                let update = self
                    .discord
                    .as_mut()
                    .map(|client| client.update_multiplayer(&map, &address, &info));
                self.handle_activity_result(update);
            }
            Err(error) => log_server_info_error(error),
        }
    }

    fn connect_discord(&mut self) {
        if self.discord.is_some() {
            return;
        }

        match DiscordRichActivityClient::connect(CLIENT_ID) {
            Ok(client) => self.discord = Some(client),
            Err(error) => eprintln!("[RPC]: connect failed: {error}"),
        }
    }

    fn update_singleplayer(&mut self, map: &str) {
        let update = self
            .discord
            .as_mut()
            .map(|client| client.update_singleplayer(map));
        self.handle_activity_result(update);
    }

    fn handle_activity_result(&mut self, result: Option<Result<(), Box<dyn Error>>>) {
        if let Some(Err(error)) = result {
            eprintln!("[RPC]: activity update failed: {error}");
            self.discord = None;
        }
    }

    fn request_server_info(&self, address: &[u8]) {
        let address = match address.to_str() {
            Ok(address) => address.to_owned(),
            Err(error) => {
                eprintln!("[RPC]: invalid UTF-8 in server address: {error}");
                return;
            }
        };

        match self.server_queries.try_send(address) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                eprintln!("[RPC]: query queue is full; skipping request");
            }
            Err(TrySendError::Disconnected(_)) => {
                eprintln!("[RPC]: query worker has stopped");
            }
        }
    }
}

fn log_server_info_error(error: ServerInfoError) {
    match error {
        ServerInfoError::IgnoredRequestError => {
            eprintln!("[RPC]: server ignored the info request");
        }
        ServerInfoError::RejectedRequestError => {
            eprintln!("[RPC]: server rejected the info request");
        }
        ServerInfoError::IoError(error) => {
            eprintln!("[RPC]: failed to query server: {error}");
        }
    }
}

fn run_event_loop(events: Receiver<AppEvent>, mut app: App) {
    while let Ok(event) = events.recv() {
        if !app.handle_event(event) {
            break;
        }
    }

    println!("BYE BYE");
}

fn main() {
    let (events_tx, events_rx) = mpsc::channel::<AppEvent>();

    spawn_log_reader(events_tx.clone());
    let server_queries = spawn_server_requester(events_tx);

    run_event_loop(events_rx, App::new(server_queries));
}
