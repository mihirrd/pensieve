use std::{fs::{File, OpenOptions}, sync::{Arc, Mutex}};
use crate::structs::AppState;
// Files
use std::io::{self, BufRead};
use std::path::Path as stdPath;
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
