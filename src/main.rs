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
use std::fs::{OpenOptions, File};
use std::io::Write;

use std::io::{self, BufRead};
use std::path::Path as stdPath;

#[derive(Serialize, Deserialize)]
struct PutRequest {
    key: String,
    val: String
}


struct AppState {
    store: store::Store,
    file: File
}


#[tokio::main(flavor = "current_thread")]
async fn main() {

     //Append-only logs
    let file_path = "op_logs.log";
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .unwrap();

    // App state to be shared among the handlers
    let store = store::Store::new(1000);
    let mut state = Arc::new(Mutex::new(AppState {
        store,
        file
    }));

    //On startup, read op_log to make the state up to date
    read_oplog(&mut state, file_path);

    // App Router
    let app = Router::new()
        .route("/get/{key}",get(store_get))
        .route("/put",post(store_put))
        .route("/delete/{key}",delete(store_delete))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


async fn store_get(
    Path(key): Path<String>,
    State(state): State<Arc<Mutex<AppState>>>)
    -> Json<Value> {

    let mut state = state.lock().unwrap();
    let val = state.store.get(key);
    Json(json!({
        "val" : val
    }))

}


async fn store_put(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<PutRequest>)
    -> impl IntoResponse {

    let key = payload.key;
    let val = payload.val;

    let mut state = state.lock().unwrap();

    state.store.put(key.clone(),val.clone());
    writeln!(state.file, "put,{},{}",key,val).unwrap();

    Json(json!({
        "status" : "ok"
    }))

}


async fn store_delete(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(key): Path<String>)
    -> Json<Value> {

    let mut state = state.lock().unwrap();
    state.store.delete(key.clone());

    writeln!(state.file, "delete,{}", key).unwrap();
    Json(json!({
        "status" : "ok"
    }))
}


fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<stdPath>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn read_oplog(state : &mut Arc<Mutex<AppState>>,file_path: &str) {
    let mut state = state.lock().unwrap();
    if let Ok(lines) = read_lines(file_path) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.map_while(Result::ok) {
            let parts: Vec<&str> = line.split(",").collect();
            match parts[0] {
                "put" => {
                    state.store.put(parts[1].to_string(), parts[2].to_string());
                },
                "delete" => {
                    state.store.delete(parts[1].to_string());
                },
                _  => {

                }
            }
        }
    }
}
