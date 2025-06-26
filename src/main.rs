use axum::{
    routing::get,
    routing::post,
    Router,
    response::Json,
};

use axum::extract::Path;
use serde_json::{Value, json};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PutRequest {
    key: String,
    val: String
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/get/{key}",get(path))
        .route("/post",post(put));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn path(Path(key): Path<u32>) -> Json<Value>{
   Json(json!({ "data": key }))
}

async fn put(Json(payload): Json<PutRequest>) -> Json<Value> {
    let key = payload.key;
    let val = payload.val;

    Json(json!({
        "key" : key,
        "val" : val
    }))

}
