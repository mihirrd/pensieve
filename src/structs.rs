use crate::store::Store;

use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Serialize, Deserialize)]
pub(crate) struct PutRequest {
    pub key: String,
    pub val: String,
}

pub enum Health{
    Alive,
    Dead
}

pub struct Peer {
    pub node_id: usize,
    pub url: String,
    pub last_seen: u64,
    pub health: Health
}

pub(crate) struct AppState {
    pub store: Store,
    pub file: File,
    pub peers: Vec<Peer>

}
