use axum::{Json, extract::Query};
use dotenv::dotenv;
use futures::TryStreamExt;
use mongodb::{
    Client, Database,
    action::InsertOne,
    bson::{doc, oid::ObjectId},
    error,
    results::InsertOneResult,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    id: ObjectId,
    email: String,
    first_name: String,
    last_name: String,
    password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoList {
    #[serde(rename = "_id")]
    id: Option<ObjectId>,
    title: String,
    owner_id: ObjectId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoItem {
    #[serde(rename = "_id")]
    id: Option<ObjectId>,
    title: String,
    description: String,
    completed: bool,
    owner_id: ObjectId,
}

#[derive(Deserialize)]
pub struct TodoQuery {
    list: ObjectId,
}

pub async fn connect() -> error::Result<Database> {
    dotenv().ok();
    match env::var("DB_URI") {
        Ok(val) => {
            let client = Client::with_uri_str(val).await?;
            match env::var("DB_NAME") {
                Ok(val) => {
                    let db = client.database(&val);
                    return Ok(db);
                }
                Err(e) => panic!("Failed to get database {}", e),
            }
        }
        Err(e) => panic!("Database connection failed {}", e),
    }
}

//CREATE
pub async fn create_list(Json(_payload): Json<TodoList>) {
    let db = connect().await.unwrap();
    let list = TodoList {
        id: Some(ObjectId::new()),
        title: _payload.title,
        owner_id: _payload.owner_id,
    };

    let _ = db
        .collection::<TodoList>("todo_lists")
        .insert_one(list)
        .await;
}

pub async fn create_todo(Json(_payload): Json<TodoItem>) {}

//READ

pub async fn get_lists() -> Json<Option<Vec<TodoList>>> {
    let db = connect().await.expect("Database connection failed");
    let lists = db.collection::<TodoList>("todo_lists").find(doc! {}).await;

    match lists {
        Ok(val) => {
            let v: Vec<TodoList> = val.try_collect().await.unwrap();
            return Json(Some(v));
        }
        Err(e) => {
            eprintln!("{e}");
            return Json(None);
        }
    }
}

pub async fn get_todos(Query(opts): Query<TodoQuery>) -> Json<Option<Vec<TodoItem>>> {
    let db = connect().await.unwrap();
    let todos = db
        .collection::<TodoItem>("todos")
        .find(doc! {
            "owner_id": opts.list
        })
        .await
        .unwrap();
    let todos = todos.try_collect().await.unwrap();
    Json(Some(todos))
}

//UPDATE

//DELETE
