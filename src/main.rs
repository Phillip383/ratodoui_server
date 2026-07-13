mod mongo_crud;

use axum::{
    Router,
    routing::{post, put},
};

#[tokio::main]
async fn main() {
    //TODO: Add the routes to the CRUD operations
    //Add one to get todos from current list...
    let app = Router::new()
        .route("/lists", post(mongo_crud::get_lists))
        .route("/todos", post(mongo_crud::get_todos))
        .route("/lists/create", put(mongo_crud::create_list))
        .route("/todos/create", put(mongo_crud::create_todo));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
