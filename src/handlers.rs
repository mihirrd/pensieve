use crate::structs::{AppState, PutRequest, Health};
use crate::util;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    response::Json,
};
use std::io::Write;
use serde_json::{Value, json};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use tracing::{event, Level};
use reqwest::Client;

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

#[derive(Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub node_id: usize,
}

#[derive(Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub status: String,
    pub node_id: usize,
}

pub async fn heartbeat_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<HeartbeatRequest>,
) -> Json<HeartbeatResponse> {
    let mut state = state.lock().unwrap();
    let current_time = util::get_unix_epoch_seconds();

    for peer in &mut state.peers {
        event!(Level::INFO, "peer node id = {}, payload node id = {}", peer.node_id, payload.node_id);
        if peer.node_id == payload.node_id {
            peer.last_seen = current_time;
            peer.health = Health::Alive;
            event!(Level::INFO, "Received heartbeat from node {}", payload.node_id);
            break;
        }
    }

    Json(HeartbeatResponse {
        status: "ok".to_string(),
        node_id: payload.node_id,
    })
}

pub async fn start_heartbeat_sender(state: Arc<Mutex<AppState>>) {
    let client = Client::new();
    let heartbeat_interval = Duration::from_secs(5);

    loop {
        sleep(heartbeat_interval).await;

        let (peers, node_id) = {
            let state_guard = state.lock().unwrap();
            (state_guard.peers.clone(), state_guard.node_config.id)
        };

        for peer in peers.iter() {
            if peer.health == Health::Alive {
                let client_clone = client.clone();
                let peer_url = peer.url.clone();
                let node_id = node_id;

                tokio::spawn(async move {
                    send_heartbeat(client_clone, node_id, peer_url).await;
                });
            }
        }
    }
}

async fn send_heartbeat(client: Client, node_id: usize, peer_url: String) {
    let heartbeat_request = HeartbeatRequest { node_id };
    let url = format!("{}/internal/heartbeat", peer_url);

    match client
        .post(&url)
        .json(&heartbeat_request)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                event!(Level::INFO, "Heartbeat sent successfully to {}", url);
            } else {
                event!(Level::WARN, "Heartbeat failed to {}: {}", url, response.status());
            }
        }
        Err(e) => {
            event!(Level::WARN, "Failed to send heartbeat to {}: {}", url, e);
        }
    }
}

pub async fn start_failure_detector(state: Arc<Mutex<AppState>>) {
    let check_interval = Duration::from_secs(10);
    let timeout_threshold = 15; // seconds

    loop {
        sleep(check_interval).await;

        let current_time = util::get_unix_epoch_seconds();
        let mut failed_nodes = Vec::new();

        {
            let mut state_guard = state.lock().unwrap();
            for peer in &mut state_guard.peers {
                if current_time - peer.last_seen > timeout_threshold {
                    if peer.health == Health::Alive {
                        peer.health = Health::Dead;
                        failed_nodes.push(peer.node_id);
                    }
                }
            }
        }

        if !failed_nodes.is_empty() {
            event!(Level::WARN, "Detected failed nodes: {:?}", failed_nodes);
        }
    }
}
