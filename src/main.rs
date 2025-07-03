mod store;
mod structs;
mod config;
mod handlers;
mod util;
use crate::structs::AppState;


/// Web framework
use axum::{
    routing::{delete, get, post}, Router
};

/// Smart pointers
use std::{env, fs::OpenOptions, sync::{Arc, Mutex}};

/// Logging
use tracing::{event, Level};
pub(crate) use tracing_subscriber;


#[tokio::main(flavor = "current_thread")]
async fn main() {

    // Initialise logging
    tracing_subscriber::fmt::init();
    event!(Level::INFO, "Initialising node");
    let node_config = config::initialise_node();
    let peers = util::parse_peer_urls(env::var("PEERS").expect("Missing PORT").to_string());

    //Append-only logs
    event!(Level::INFO, "Initialising oplogs");
    let file_path = "./op_logs.log".to_string();

    let file_result = OpenOptions::new().create(true).append(true).open(&file_path);

    let file = match file_result {
        Ok(file) => file,
        Err(_error) => {
            event!(Level::ERROR, "Could not load oplogs");
            panic!("Shutting down");
        }
    };

    event!(Level::INFO, "Loading state vars");
    // App state to be shared among the handlers
    // TODO: Refactor cache size allocation
    let store = store::Store::new(1000);
    let mut state = Arc::new(Mutex::new(AppState { store, file, peers }));

    //On startup, read op_log to make the state up to date
    util::read_oplog(&mut state, &file_path);

    // App Router
    let app = Router::new()
        .route("/get/{key}", get(handlers::store_get))
        .route("/put", post(handlers::store_put))
        .route("/delete/{key}", delete(handlers::store_delete))
        .with_state(state);

    event!(Level::INFO, "Starting up the server...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    event!(Level::INFO, "Server running on port: {0:?}", node_config.port);
    axum::serve(listener, app).await.unwrap();
}
