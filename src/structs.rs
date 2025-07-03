use crate::{config::Node, store::Store};

use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Serialize, Deserialize)]
pub(crate) struct PutRequest {
    pub key: String,
    pub val: String,
}

#[derive(Clone, PartialEq)]
pub enum Health{
    Alive,
    Dead
}

#[derive(Clone)]
pub struct Peer {
    pub node_id: usize,
    pub url: String,
    pub last_seen: u64,
    pub health: Health
}

pub(crate) struct AppState {
    pub node_config: Node,
    pub store: Store,
    pub file: File,
    pub peers: Vec<Peer>

}
