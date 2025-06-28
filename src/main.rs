mod store;
mod structs;
use crate::structs::{AppState, PutRequest};

// Web framework
use axum::{
    Router,
    extract::{Path, State},
    response::IntoResponse,
    response::Json,
    routing::delete,
    routing::get,
    routing::post,
};

// Serialization
use serde_json::{Value, json};

// Smart pointers
use std::sync::{Arc, Mutex};

// Files
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::io::{self, BufRead};
use std::path::Path as stdPath;

// Logging
use tracing::{event, Level};
use tracing_subscriber;

use std::env;


#[tokio::main(flavor = "current_thread")]
async fn main() {

    // Initialise logging
    tracing_subscriber::fmt::init();
    event!(Level::INFO, "Initialising node");

    let node_id: u32 = env::var("NODE_ID")
        .expect("NODE_ID not set")
        .parse()
        .expect("NODE_ID must be a valid number");

    event!(Level::INFO, "Starting node with ID: {}", node_id);

    //Append-only logs
    event!(Level::INFO, "Initialising oplogs");
    let file_path = "./op_logs.log";
    let file_result = OpenOptions::new().create(true).append(true).open(file_path);

    let file = match file_result {
        Ok(file) => file,
        Err(_error) => {
            event!(Level::ERROR, "Could not load oplogs");
            panic!("Shutting down");
        }
    };

    event!(Level::INFO, "Loading state vars");
    // App state to be shared among the handlers
    let store = store::Store::new(1000);
    let mut state = Arc::new(Mutex::new(AppState { store, file }));

    //On startup, read op_log to make the state up to date
    read_oplog(&mut state, file_path);

    // App Router
    let app = Router::new()
        .route("/get/{key}", get(store_get))
        .route("/put", post(store_put))
        .route("/delete/{key}", delete(store_delete))
        .with_state(state);

    event!(Level::INFO, "Starting up the server...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    event!(Level::INFO, "Server running on port 7878");
    axum::serve(listener, app).await.unwrap();
}

async fn store_get(
    Path(key): Path<String>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<Value> {
    let mut state = state.lock().unwrap();
    let val = state.store.get(key);
    Json(json!({
        "val" : val
    }))
}

async fn store_put(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<PutRequest>,
) -> impl IntoResponse {
    let key = payload.key;
    let val = payload.val;

    let mut state = state.lock().unwrap();

    state.store.put(key.clone(), val.clone());
    writeln!(state.file, "put,{},{}", key, val).unwrap();

    Json(json!({
        "status" : "ok"
    }))
}

async fn store_delete(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(key): Path<String>,
) -> Json<Value> {
    let mut state = state.lock().unwrap();
    state.store.delete(key.clone());

    writeln!(state.file, "delete,{}", key).unwrap();
    Json(json!({
        "status" : "ok"
    }))
}

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

fn read_oplog(state: &mut Arc<Mutex<AppState>>, file_path: &str) {
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
