use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
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

use crate::auth::{self, hash_pass};

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
pub struct NewUser {
    #[serde(rename = "_id")]
    id: Option<ObjectId>,
    email: String,
    first_name: String,
    last_name: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoList {
    #[serde(rename = "_id")]
    id: Option<ObjectId>,
    title: String,
    owner_id: Option<ObjectId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoItem {
    #[serde(rename = "_id")]
    id: Option<ObjectId>,
    title: String,
    description: String,
    completed: bool,
    owner_id: Option<ObjectId>,
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

#[derive(Deserialize)]
pub struct UserQuery {
    id: ObjectId,
}

#[derive(Deserialize)]
pub struct LoginQuery {
    email: String,
    password: String,
}

pub async fn connect() -> error::Result<Database> {
    dotenv().ok();
    let uri = env::var("DB_URI").expect("DB_URI must be set");
    let db_name = env::var("DB_NAME").expect("DB_NAME must be set");
    let client = Client::with_uri_str(uri).await?;
    Ok(client.database(db_name.as_str()))
}

//TODO: Ensure a user can only access data they own at every endpoint!
//TODO: Add status code returns to every endpoint! This will allow for frontend error messages!

//####### User endpoints #####

//New account

pub async fn create_account(
    State(db): State<Database>,
    Json(payload): Json<NewUser>,
) -> Result<StatusCode, StatusCode> {
    let hash = hash_pass(payload.password.as_str());

    //TODO: Handle emails that already have an account...
    match hash {
        Ok(val) => {
            let new_user = User {
                id: ObjectId::new(),
                email: payload.email,
                first_name: payload.first_name,
                last_name: payload.last_name,
                password_hash: val,
            };
            let update = db.collection::<User>("users").insert_one(new_user).await;
            match update {
                Ok(_val) => return Ok(StatusCode::OK), //TODO: auto login here?
                Err(_e) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_e) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

//login
pub async fn login(
    State(db): State<Database>,
    Json(payload): Json<LoginQuery>,
) -> Result<StatusCode, StatusCode> {
    let user = db
        .collection::<User>("users")
        .find_one(doc! {"email": payload.email})
        .await;

    let user_opt = match user {
        Ok(val) => val,
        Err(_e) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match user_opt {
        Some(user) => {
            if auth::verify_hash(&user.password_hash, &payload.password) {
                return Ok(StatusCode::OK);
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        None => return Err(StatusCode::NOT_FOUND),
    }
}

//Update User

pub async fn update_user(State(db): State<Database>, Json(payload): Json<User>) {}

//Delete account

pub async fn delete_user(State(db): State<Database>, Query(payload): Query<UserQuery>) {
    let _ = db
        .collection::<User>("users")
        .delete_one(doc! {
            "_id": payload.id
        })
        .await;
}

//######## List and Todo endpoints... #########

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
//TODO: Refactor these to only require the specific parts needing updates instead of the entire doc...
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
