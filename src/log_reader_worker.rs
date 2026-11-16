use bstr::io::BufReadExt;
use std::{io, sync::mpsc, thread};

use crate::AppEvent;

const TAG: &str = "[DRPC]:log_reader:";

pub fn spawn_log_reader_worker(events: mpsc::Sender<AppEvent>) {
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
                    eprintln!("{TAG} Failed to read log input: {error}");
                    break;
                }
            }
        }

        let _ = events.send(AppEvent::LogEnded);
    });
}
