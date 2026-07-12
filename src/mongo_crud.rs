use axum::{Json, extract::Query};
use dotenv::dotenv;
use futures::TryStreamExt;
use mongodb::{
    Client, Collection, Cursor, Database,
    bson::{Document, doc, from_document, oid::ObjectId},
    error,
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
    id: ObjectId,
    title: String,
    owner_id: ObjectId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoItem {
    #[serde(rename = "_id")]
    id: ObjectId,
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

pub async fn get_list_by_name(name: &str) -> Option<Document> {
    let db = connect().await.unwrap();
    let list = db
        .collection("todo_lists")
        .find_one(doc! {"title" : name})
        .await
        .unwrap_or(None);

    println!("{:?}", list);
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn get_test_mongo() -> Database {
        connect().await.expect("Failed to connect to MongoDB Atlas")
    }

    #[tokio::test]
    async fn test_connect() {
        let mongo = get_test_mongo().await;
    }

    #[tokio::test]
    async fn test_insert_list() {
        let mongo = get_test_mongo().await;
    }

    #[tokio::test]
    async fn test_insert_todo() {
        //insert todo
        //query for inserted todo
        //assert if todo exists in list and todos collection
    }

    #[tokio::test]
    async fn test_delete_list() {
        //insert list
        //delete list
        //query for list
        //assert if list exist
    }

    #[tokio::test]
    async fn test_delete_todo() {
        //insert todo
        //delete todo
        //query for todo
        //assert if todo found
    }

    #[tokio::test]
    async fn test_update_list() {
        //insert list
        //update list title
        //query for list
        //assert if title eqauls update
    }

    #[tokio::test]
    async fn test_update_todo() {
        //insert todo
        //update todo title and description
        //query for todo
        //assert if title and description equals update
    }

    #[tokio::test]
    async fn test_lists() {
        let lists = get_lists().await;
        assert!(lists.is_some());
    }

    #[tokio::test]
    async fn test_get_list() {
        let list = get_list_by_name("General").await;
        assert!(list.is_some());
    }
}
