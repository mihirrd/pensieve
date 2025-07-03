use crate::structs::{AppState, PutRequest};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    response::Json,
};
use std::io::Write;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

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
