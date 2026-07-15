use axum::{
    Json,
    extract::{Query, State},
};
use dotenv::dotenv;
use futures::TryStreamExt;
use mongodb::{
    Client, Database,
    bson::{doc, oid::ObjectId},
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
    list: Option<ObjectId>,
    id: Option<ObjectId>,
}

#[derive(Deserialize)]
pub struct ListQuery {
    id: ObjectId,
}

pub async fn connect() -> error::Result<Database> {
    dotenv().ok();
    let uri = env::var("DB_URI").expect("DB_URI must be set");
    let db_name = env::var("DB_NAME").expect("DB_NAME must be set");
    let client = Client::with_uri_str(uri).await?;
    Ok(client.database(db_name.as_str()))
}

//CREATE
pub async fn create_list(State(db): State<Database>, Json(_payload): Json<TodoList>) {
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

pub async fn create_todo(State(db): State<Database>, Json(_payload): Json<TodoItem>) {
    let todo = TodoItem {
        id: Some(ObjectId::new()),
        title: _payload.title,
        description: _payload.description,
        completed: _payload.completed,
        owner_id: _payload.owner_id,
    };

    let _ = db.collection::<TodoItem>("todos").insert_one(todo).await;
}

//READ

pub async fn get_lists(State(db): State<Database>) -> Json<Option<Vec<TodoList>>> {
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

pub async fn get_todos(
    State(db): State<Database>,
    Query(opts): Query<TodoQuery>,
) -> Json<Option<Vec<TodoItem>>> {
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

pub async fn update_list(State(db): State<Database>, Json(_payload): Json<TodoList>) {
    let _ = db
        .collection::<TodoList>("todo_lists")
        .update_one(
            doc! {"_id": _payload.id},
            doc! {
                "$set": {
                    "title": _payload.title
                }
            },
        )
        .await;
}

pub async fn update_todo(State(db): State<Database>, Json(_payload): Json<TodoItem>) {
    let _ = db
        .collection::<TodoItem>("todos")
        .update_one(
            doc! {
                "_id": _payload.id
            },
            doc! {
                "$set": {
                    "title": _payload.title,
                    "description": _payload.description,
                    "completed": _payload.completed,
                    "owner_id": _payload.owner_id,
                }
            },
        )
        .await;
}

//DELETE

pub async fn remove_list(State(db): State<Database>, Query(_opts): Query<ListQuery>) {
    //Remove the list..
    let result = db
        .collection::<TodoList>("todo_lists")
        .delete_one(doc! {
            "_id": _opts.id
        })
        .await
        .unwrap_or_default();

    //Only delete todo's if list was deleted.
    if result.deleted_count > 0 {
        //Delete the todo's in the list...
        let _ = db
            .collection::<TodoItem>("todos")
            .delete_many(doc! {"owner_id": _opts.id})
            .await;
    }
}

pub async fn remove_todo(State(db): State<Database>, Query(_opts): Query<TodoQuery>) {
    let _ = db
        .collection::<TodoItem>("todos")
        .delete_one(doc! {
            "_id": _opts.id,
        })
        .await;
}
