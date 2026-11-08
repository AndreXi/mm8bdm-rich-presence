mod discord_rich_activity;
mod log_line_parser;
mod log_reader_worker;
mod map_name;
mod server;
mod server_worker;

use bstr::ByteSlice;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};

use discord_rich_presence::error::Error as DiscordIPCError;

use crate::{
    discord_rich_activity::DiscordRichActivityClient,
    log_line_parser::{GameEvent, log_line_parser},
    log_reader_worker::spawn_log_reader_worker,
    map_name::build_map_name,
    server::ServerResponseData,
    server_worker::spawn_server_worker,
};

const CLIENT_ID: &str = "1561172546756280430";

pub enum AppEvent {
    LogLine(Vec<u8>),
    LogEnded,
    ServerInfoUpdated {
        address: String,
        result: ServerResponseData,
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

    fn handle_server_info(&mut self, address: String, result: ServerResponseData) {
        let map = match &self.session {
            Session::Multiplayer {
                address: current_address,
                map: Some(map),
            } if current_address.to_str().ok() == Some(address.as_str()) => map.clone(),
            _ => return,
        };

        self.connect_discord();
        let update = self
            .discord
            .as_mut()
            .map(|client| client.update_multiplayer(&map, &address, &result));
        self.handle_activity_result(update);
    }

    fn connect_discord(&mut self) {
        if self.discord.is_some() {
            return;
        }

        match DiscordRichActivityClient::connect(CLIENT_ID) {
            Ok(client) => self.discord = Some(client),
            Err(error) => eprintln!("[DRPC]: connect failed: {error}"),
        }
    }

    fn update_singleplayer(&mut self, map: &str) {
        let update = self
            .discord
            .as_mut()
            .map(|client| client.update_singleplayer(map));
        self.handle_activity_result(update);
    }

    fn handle_activity_result(&mut self, result: Option<Result<(), DiscordIPCError>>) {
        if let Some(Err(error)) = result {
            eprintln!("[DRPC]: activity update failed: {error}");
            self.discord = None;
        }
    }

    fn request_server_info(&self, address: &[u8]) {
        let address = match address.to_str() {
            Ok(address) => address.to_owned(),
            Err(error) => {
                eprintln!("[DRPC]: invalid UTF-8 in server address: {error}");
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

    spawn_log_reader_worker(events_tx.clone());
    let server_queries = spawn_server_worker(events_tx);

    run_event_loop(events_rx, App::new(server_queries));
}
