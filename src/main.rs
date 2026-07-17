mod auth;
mod mongo_crud;

use axum::{
    Router,
    routing::{delete, get, post, put},
};

#[tokio::main]
async fn main() {
    let db = mongo_crud::connect()
        .await
        .expect("Database connection failed.");

    let app = Router::new()
        .route("/lists", get(mongo_crud::get_lists))
        .route("/todos", get(mongo_crud::get_todos))
        .route("/lists/create", put(mongo_crud::create_list))
        .route("/todos/create", put(mongo_crud::create_todo))
        .route("/lists/delete", delete(mongo_crud::remove_list))
        .route("/todos/delete", delete(mongo_crud::remove_todo))
        .route("/lists/update", put(mongo_crud::update_list))
        .route("/todos/update", put(mongo_crud::update_todo))
        .route("/users/create", put(mongo_crud::create_account))
        .route("/users/login", post(mongo_crud::login))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
