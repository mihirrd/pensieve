use std::{fs::{File}, sync::{Arc, Mutex}};
use crate::structs::{AppState, Peer, Health};
// Files
use std::io::{self, BufRead};
use std::path::Path as stdPath;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{event, Level};

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<stdPath>,
{
    File::open(&filename)
        .map_err(|e| {
            event!(
                Level::WARN,
                "Failed to read oplogs {:?}: {}",
                filename.as_ref(),
                e
            );
            e
        })
        .map(|file| io::BufReader::new(file).lines())
}


pub fn read_oplog(state: &mut Arc<Mutex<AppState>>, file_path: &str) {

    let mut state = state.lock().unwrap();
    if let Ok(lines) = read_lines(file_path) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.map_while(Result::ok) {
            let parts: Vec<&str> = line.split(",").collect();
            match parts[0] {
                "put" => {
                    state.store.put(parts[1].to_string(), parts[2].to_string());
                }
                "delete" => {
                    state.store.delete(parts[1].to_string());
                }
                _ => {}
            }
        }
    }
}


pub fn get_unix_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub fn parse_peer_urls(peer_urls_str: String) -> Vec<Peer> {
    peer_urls_str
        .split(',')
        .filter_map(|url| {
            let url = url.trim();
            if url.is_empty() {
                return None;
            }

            // Extract node ID from URL (e.g., "https://node1:7878" -> "node1")
            let node_id = url
                .split("://")
                .nth(1)?
                .split(':')
                .next()?
                .to_string();

            // Convert node name to numeric ID (e.g., "node1" -> 1)
            let numeric_id = node_id
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<usize>()
                .unwrap_or(0);

            Some(Peer {
                node_id: numeric_id,
                url: url.to_string(),
                last_seen: get_unix_epoch_seconds(),
                health: Health::Alive,
            })
        })
        .collect()
}
