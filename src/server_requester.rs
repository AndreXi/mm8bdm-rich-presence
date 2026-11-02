use std::{
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

use crate::{AppEvent, server_info::get_server_info};

const SERVER_INFO_REQUEST_INTERVAL: Duration = Duration::from_secs(60);

pub fn spawn_server_requester(events: mpsc::Sender<AppEvent>) -> mpsc::SyncSender<String> {
    let (query_tx, query_rx) = mpsc::sync_channel::<String>(1);

    thread::spawn(move || {
        let Ok(mut address) = query_rx.recv() else {
            return;
        };

        loop {
            let request_started = Instant::now();
            let result = get_server_info(&address);

            if events
                .send(AppEvent::ServerInfoUpdated {
                    address: address.clone(),
                    result,
                })
                .is_err()
            {
                break;
            }

            let next_request_at = request_started + SERVER_INFO_REQUEST_INTERVAL;
            if !wait_for_next_request(&query_rx, next_request_at, &mut address) {
                break;
            }
        }
    });

    query_tx
}

fn wait_for_next_request(
    query_rx: &Receiver<String>,
    next_request_at: Instant,
    last_address: &mut String,
) -> bool {
    loop {
        let remaining = next_request_at.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return true;
        }

        match query_rx.recv_timeout(remaining) {
            Ok(address) => *last_address = address,
            Err(RecvTimeoutError::Timeout) => return true,
            Err(RecvTimeoutError::Disconnected) => return false,
        }
    }
}
