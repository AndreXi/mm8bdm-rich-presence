use bstr::io::BufReadExt;
use std::{io, sync::mpsc, thread};

use crate::AppEvent;

pub fn spawn_log_reader(events: mpsc::Sender<AppEvent>) {
    thread::spawn(move || {
        let stdin = io::stdin();

        for line in stdin.lock().byte_lines() {
            match line {
                Ok(line) => {
                    if events.send(AppEvent::LogLine(line.to_vec())).is_err() {
                        break; // Main loop has ended.
                    }
                }
                Err(error) => {
                    eprintln!("Failed to read log input: {error}");
                    break;
                }
            }
        }

        let _ = events.send(AppEvent::LogEnded);
    });
}
