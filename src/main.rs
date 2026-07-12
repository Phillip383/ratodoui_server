mod mongo_crud;

use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    //TODO: Add the routes to the CRUD operations
    let app = Router::new();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
