use crate::structs::{AppState, PutRequest, HearbeatPong};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    response::Json,
};
use tokio::time;
use reqwest::Client;
use std::io::Write;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tracing::{event, Level};


pub async fn store_get(
    Path(key): Path<String>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<Value> {
    let mut state = state.lock().unwrap();
    let val = state.store.get(key);
    Json(json!({
        "val" : val
    }))
}


pub async fn store_put(
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


pub async fn store_delete(
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

pub async fn heartbeat_pong(State(state): State<Arc<Mutex<AppState>>>)
 -> Json<HearbeatPong> {

    let state = state.lock().unwrap();
    let node_id = &state.node.id;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();


    let pong = HearbeatPong {
        node_id: node_id.to_string(),
        timestamp: timestamp,
        status: "ok".to_string()
    };

    Json(pong)
}

async fn heartbeat_ping(url: String, interval_secs: u64) {
      let client = Client::new();
      let mut interval = time::interval(Duration::from_secs(interval_secs));

      loop {
          interval.tick().await;

          let response =  client.get(url.clone()).send().await;

          match response {
              Ok(resp) => {
                  if resp.status().is_success() {
                      event!(Level::DEBUG, "Successfully called endpoint: {}", url);
                  } else {
                      event!(Level::WARN, "Endpoint call failed: {} - Status: {}", url, resp.status());
                  }
              }
              Err(e) => {
                  event!(Level::ERROR, "Failed to call endpoint {}: {}", url, e);
              }
          }
      }
}

pub async fn start_heartbeats(state: Arc<Mutex<AppState>>) {
    let state = state.lock().unwrap();
    let peer_urls = state.node.peers.clone();

    for url in peer_urls {
        heartbeat_ping(url, 5);
    }

}
