use crate::store::Store;

use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Serialize, Deserialize)]
pub(crate) struct PutRequest {
    pub key: String,
    pub val: String,
}

pub(crate) struct AppState {
    pub store: Store,
    pub file: File,
}
