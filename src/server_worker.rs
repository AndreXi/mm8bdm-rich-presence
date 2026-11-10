use std::{
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

use crate::{
    AppEvent,
    server::{
        MAX_CODEC_OUTPUT_SIZE, MAX_NETWORK_BUFFER_SIZE, ServerRequestError, fetch_server_info,
    },
};

const SERVER_INFO_REQUEST_INTERVAL: Duration = Duration::from_secs(60);

pub fn spawn_server_worker(events: mpsc::Sender<AppEvent>) -> mpsc::SyncSender<String> {
    let (query_tx, query_rx) = mpsc::sync_channel::<String>(1);

    thread::spawn(move || {
        let Ok(mut address) = query_rx.recv() else {
            return;
        };

        let mut network_buffer = [0u8; MAX_NETWORK_BUFFER_SIZE];
        let mut codec_buffer = Vec::with_capacity(MAX_CODEC_OUTPUT_SIZE);

        loop {
            let request_started = Instant::now();
            let result = fetch_server_info(&address, &mut network_buffer, &mut codec_buffer);

            match result {
                Err(error) => match error {
                    ServerRequestError::CannotBindSocket => {
                        println!("[DRPC]:server_worker: Unable to bind socket / IO Error.");
                        break;
                    }
                    ServerRequestError::BannedRequestError => {
                        println!("[DRPC]:server_worker: You are banned from the server.");
                        break;
                    }
                    ServerRequestError::CodecBufferLimitReached => {
                        println!("[DRPC]:server_worker: Received packet was too big.");
                        break;
                    }
                    ServerRequestError::UnknownRequestError => {
                        println!("[DRPC]:server_worker: Unknown server response.");
                        break;
                    }
                    ServerRequestError::IgnoredRequestError => {
                        // TODO: It should increment the polling rate duration and stop after some limits
                        println!("[DRPC]:server_worker: Server ignored the request.");
                    }
                },
                Ok(None) => {
                    println!("[DRPC]:server_worker: Server response parsed twice!");
                    debug_assert!(false);
                    break;
                }
                Ok(Some(value)) => {
                    if events
                        .send(AppEvent::ServerInfoUpdated {
                            address: address.clone(),
                            result: value,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            }

            let next_request_at = request_started + SERVER_INFO_REQUEST_INTERVAL;
            if !wait_for_next_request(&query_rx, next_request_at, &mut address) {
                break;
            }
        }

        println!("[DRPC]:server_worker: exit");
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
