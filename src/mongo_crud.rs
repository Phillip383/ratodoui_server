use axum::{
    Json,
    extract::{FromRequestParts, Query, State},
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
};
use chrono::{Duration, Utc};
use dotenv::dotenv;
use futures::TryStreamExt;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
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
    list_id: Option<ObjectId>,
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
pub struct LoginQuery {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|val| val.to_str().ok());

        let auth_header = match auth_header {
            Some(header) => header,
            None => return Err(StatusCode::UNAUTHORIZED),
        };

        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = &auth_header[7..];
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(token_data.claims)
    }
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
) -> Result<Json<AuthResponse>, StatusCode> {
    let user = db
        .collection::<User>("users")
        .find_one(doc! {"email": payload.email})
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(user) => {
            if auth::verify_hash(&user.password_hash, &payload.password) {
                let exp = Utc::now()
                    .checked_add_signed(Duration::hours(24))
                    .expect("valid timestamp")
                    .timestamp() as usize;

                let claims = Claims {
                    sub: user.id.to_hex(),
                    exp,
                };

                let secret = env::var("JWT_SECRET").expect("JWT_SECRET not set");
                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(secret.as_bytes()),
                )
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                return Ok(Json(AuthResponse { token }));
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        None => return Err(StatusCode::NOT_FOUND),
    }
}

//Update User

pub async fn update_user(
    _claims: Claims,
    State(_db): State<Database>,
    Json(_payload): Json<User>,
) -> Result<StatusCode, StatusCode> {
    //TODO: Implement update user
    Ok(StatusCode::OK)
}

//Delete account

pub async fn delete_user(
    claims: Claims,
    State(db): State<Database>,
) -> Result<StatusCode, StatusCode> {
    let user_id = claims
        .sub
        .parse::<ObjectId>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    match db
        .collection::<User>("users")
        .delete_one(doc! {
            "_id": user_id
        })
        .await
    {
        Ok(res) => {
            if res.deleted_count > 0 {
                Ok(StatusCode::OK)
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

//######## List and Todo endpoints... #########

//CREATE
pub async fn create_list(
    claims: Claims,
    State(db): State<Database>,
    Json(_payload): Json<TodoList>,
) -> Result<StatusCode, StatusCode> {
    let owner_id = claims
        .sub
        .parse::<ObjectId>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let list = TodoList {
        id: Some(ObjectId::new()),
        title: _payload.title,
        owner_id: Some(owner_id),
    };

    match db
        .collection::<TodoList>("todo_lists")
        .insert_one(list)
        .await
    {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_todo(
    claims: Claims,
    State(db): State<Database>,
    Json(_payload): Json<TodoItem>,
) -> Result<StatusCode, StatusCode> {
    let owner_id = claims
        .sub
        .parse::<ObjectId>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let todo = TodoItem {
        id: Some(ObjectId::new()),
        title: _payload.title,
        description: _payload.description,
        completed: _payload.completed,
        list_id: _payload.list_id, //TODO: This is probably not the best way to handle this, should check if the list exists before creating the todo and owned by the user...
        owner_id: Some(owner_id),
    };

    match db.collection::<TodoItem>("todos").insert_one(todo).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

//READ

pub async fn get_lists(claims: Claims, State(db): State<Database>) -> Json<Option<Vec<TodoList>>> {
    let owner_id = claims
        .sub
        .parse::<ObjectId>()
        .map_err(|_| StatusCode::BAD_REQUEST)
        .ok();

    let lists = db
        .collection::<TodoList>("todo_lists")
        .find(doc! {
            "owner_id": owner_id
        })
        .await;

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
    claims: Claims,
    State(db): State<Database>,
    Query(qry): Query<TodoQuery>,
) -> Json<Option<Vec<TodoItem>>> {
    let owner_id = claims
        .sub
        .parse::<ObjectId>()
        .map_err(|_| StatusCode::BAD_REQUEST)
        .ok();

    let todos = db
        .collection::<TodoItem>("todos")
        .find(doc! {
            "owner_id": owner_id,
            "list_id": qry.list
        })
        .await
        .unwrap();
    let todos = todos.try_collect().await.unwrap();
    Json(Some(todos))
}

//UPDATE
//TODO: Refactor these to only require the specific parts needing updates instead of the entire doc...
pub async fn update_list(
    claims: Claims,
    State(db): State<Database>,
    Json(_payload): Json<TodoList>,
) -> Result<StatusCode, StatusCode> {
    let owner_id = claims
        .sub
        .parse::<ObjectId>()
        .map_err(|_| StatusCode::BAD_REQUEST)
        .ok();

    match db
        .collection::<TodoList>("todo_lists")
        .update_one(
            doc! {"_id": _payload.id, "owner_id": owner_id},
            doc! {
                "$set": {
                    "title": _payload.title
                }
            },
        )
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_todo(
    claims: Claims,
    State(db): State<Database>,
    Json(_payload): Json<TodoItem>,
) -> Result<StatusCode, StatusCode> {
    let owner_id = claims
        .sub
        .parse::<ObjectId>()
        .map_err(|_| StatusCode::BAD_REQUEST)
        .ok();

    match db
        .collection::<TodoItem>("todos")
        .update_one(
            doc! {
                "_id": _payload.id,
                "owner_id": owner_id
            },
            doc! {
                "$set": {
                    "title": _payload.title,
                    "description": _payload.description,
                    "completed": _payload.completed,
                }
            },
        )
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

//DELETE

pub async fn remove_list(
    claims: Claims,
    State(db): State<Database>,
    Query(_opts): Query<ListQuery>,
) -> Result<StatusCode, StatusCode> {
    let owner_id = claims
        .sub
        .parse::<ObjectId>()
        .map_err(|_| StatusCode::BAD_REQUEST)
        .ok();

    match db
        .collection::<TodoList>("todo_lists")
        .delete_one(doc! {
            "_id": _opts.id,
            "owner_id": owner_id
        })
        .await
    {
        Ok(result) => {
            if result.deleted_count > 0 {
                match db
                    .collection::<TodoItem>("todos")
                    .delete_many(doc! {"owner_id": _opts.id})
                    .await
                {
                    Ok(_) => return Ok(StatusCode::OK),
                    Err(_) => {
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
            } else {
                return Ok(StatusCode::NOT_FOUND);
            }
        }
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

pub async fn remove_todo(
    claims: Claims,
    State(db): State<Database>,
    Query(_opts): Query<TodoQuery>,
) -> Result<StatusCode, StatusCode> {
    let owner_id = claims
        .sub
        .parse::<ObjectId>()
        .map_err(|_| StatusCode::BAD_REQUEST)
        .ok();

    match db
        .collection::<TodoItem>("todos")
        .delete_one(doc! {
            "_id": _opts.id,
            "owner_id": owner_id
        })
        .await
    {
        Ok(result) => {
            if result.deleted_count > 0 {
                return Ok(StatusCode::OK);
            } else {
                return Ok(StatusCode::NOT_FOUND);
            }
        }
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}
