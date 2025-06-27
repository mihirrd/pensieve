mod store;
use axum::{
    routing::get,
    routing::post,
    routing::delete,
    extract::{Path,State},
    response::IntoResponse,
    Router,
    response::Json,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
struct PutRequest {
    key: String,
    val: String
}


struct AppState {
    store: store::Store
}


#[tokio::main(flavor = "current_thread")]
async fn main() {

    let store = store::Store::new(1000);
    let state = Arc::new(Mutex::new(AppState {store}));

    let app = Router::new()
        .route("/get/{key}",get(store_get))
        .route("/put",post(store_put))
        .route("/delete/{key}",delete(store_delete))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


async fn store_get(Path(key): Path<String>,State(state): State<Arc<Mutex<AppState>>>) -> Json<Value>{
    let mut state = state.lock().unwrap();
    let val = state.store.get(key);
    Json(json!({
        "val" : val
    }))

}


async fn store_put(State(state): State<Arc<Mutex<AppState>>>, Json(payload): Json<PutRequest>)
                   -> impl IntoResponse {
    let key = payload.key;
    let val = payload.val;

    let mut state = state.lock().unwrap();
    state.store.put(key,val);

    Json(json!({
        "status" : "ok"
    }))

}


async fn store_delete(State(state): State<Arc<Mutex<AppState>>>, Path(key): Path<String>) -> Json<Value> {
    let mut state = state.lock().unwrap();
    state.store.delete(key);
    Json(json!({
        "status" : "ok"
    }))
}
