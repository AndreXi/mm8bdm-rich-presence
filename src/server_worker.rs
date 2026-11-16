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

const TAG: &str = "[DRPC]:server_worker:";

pub fn spawn_server_worker(events: mpsc::Sender<AppEvent>) -> mpsc::SyncSender<String> {
    let (query_tx, query_rx) = mpsc::sync_channel::<String>(1);

    let mut timeout_duration: Duration = Duration::from_secs(60);

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
                        println!("{TAG} Unable to bind socket / IO Error.");
                        break;
                    }
                    ServerRequestError::BannedRequestError => {
                        println!("{TAG} You are banned from the server.");
                        break;
                    }
                    ServerRequestError::CodecBufferLimitReached => {
                        println!("{TAG} Received packet was too big.");
                        break;
                    }
                    ServerRequestError::UnknownRequestError => {
                        println!("{TAG} Unknown server response.");
                        break;
                    }
                    ServerRequestError::IgnoredRequestError => {
                        if timeout_duration >= Duration::from_mins(5) {
                            eprintln!("{TAG} Server ignored too many times!");
                            break;
                        }
                        timeout_duration += Duration::from_secs(60);
                        println!(
                            "{TAG} Server ignored the request. next retry in: {:#?}",
                            timeout_duration
                        );
                    }
                },
                Ok(None) => {
                    println!("{TAG} Server response parsed twice!");
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

            let next_request_at = request_started + timeout_duration;
            if !wait_for_next_request(&query_rx, next_request_at, &mut address) {
                break;
            }
        }

        println!("{TAG} exit");
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
