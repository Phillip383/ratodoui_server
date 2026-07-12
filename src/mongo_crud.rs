use dotenv::dotenv;
use mongodb::{
    Client, Collection, Cursor, Database,
    bson::{Document, doc, from_document, oid::ObjectId},
    error,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct TodoList {
    #[serde(rename = "_id")]
    id: ObjectId,
    title: String,
    todo_items: Vec<TodoItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TodoItem {
    #[serde(rename = "_id")]
    id: ObjectId,
    title: String,
    description: String,
    completed: bool,
}

struct MongoCrud {
    db: Database,
}

impl MongoCrud {
    pub async fn connect() -> error::Result<MongoCrud> {
        dotenv().ok();
        match env::var("DB_URI") {
            Ok(val) => {
                let client = Client::with_uri_str(val).await?;
                match env::var("DB_NAME") {
                    Ok(val) => {
                        let db = client.database(&val);
                        let mongo = MongoCrud { db };
                        return Ok(mongo);
                    }
                    Err(e) => panic!("Failed to get database {}", e),
                }
            }
            Err(e) => panic!("Database connection failed {}", e),
        }
    }

    pub async fn insert_list(&self, list: TodoList) {}

    pub async fn insert_todo(&self, todo: TodoItem) {}

    pub async fn remove_list(&self, id: ObjectId) {}

    pub async fn remove_todo(&self, id: ObjectId) {}

    pub async fn update_list(&self, list: TodoList) {}

    pub async fn update_todo(&self, todo: TodoItem) {}

    pub async fn get_list_by_name(&self, name: &str) -> Option<Document> {
        let list = self
            .db
            .collection("todo_lists")
            .find_one(doc! {"title" : name})
            .await
            .unwrap_or(None);

        println!("{:?}", list);
        list
    }

    pub async fn get_todos(&self, list_name: &str) -> error::Result<TodoList> {
        let pipeline = vec![
            doc! {
                "$match": {
                    "title": list_name,
                }
            },
            doc! {
            "$lookup": {
                "from": "todos",
                "localField": "todos",
                "foreignField": "_id",
                "as": "todo_items",
            }},
            doc! {
                "$project": {
                    "title": 1,
                    "todo_items": 1
                }
            },
        ];
        let col: Collection<Document> = self.db.collection("todo_lists");
        let mut cursor = col.aggregate(pipeline).await?;

        while cursor.advance().await? {
            let doc = cursor.deserialize_current()?;
            let list: TodoList = from_document(doc)?;
            println!("{:?}", list);
            return Ok(list);
        }
        Err(error::Error::custom("Failed to populate todo lists"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn get_test_mongo() -> MongoCrud {
        MongoCrud::connect()
            .await
            .expect("Failed to connect to MongoDB Atlas")
    }

    #[tokio::test]
    async fn test_connect() {
        let mongo = get_test_mongo().await;
    }

    #[tokio::test]
    async fn test_insert_list() {
        let mongo = get_test_mongo().await;

        //insert list
        let list = TodoList {
            id: ObjectId::new(),
            title: "Default".to_string(),
            todo_items: Vec::new(),
        };

        let result = mongo
            .db
            .collection::<TodoList>("todo_lists")
            .insert_one(list)
            .await
            .unwrap();

        //query for inserted list
        let query = mongo
            .db
            .collection::<TodoList>("todo_lists")
            .find_one(doc! {"_id": result.inserted_id})
            .await
            .unwrap();

        //assert if list exists
        assert!(query.is_some());

        //TODO: Clean up test insertion...
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
        let mongo = get_test_mongo().await;
        let lists: mongodb::Collection<Document> = mongo.db.collection("todo_lists");

        assert!(lists.name() == "todo_lists");
    }

    #[tokio::test]
    async fn test_get_list() {
        let mongo = get_test_mongo().await;
        let list = mongo.get_list_by_name("General").await;
        assert!(list.is_some());
    }

    #[tokio::test]
    async fn test_get_todos() {
        let mongo = get_test_mongo().await;
        let todo_id = mongo
            .db
            .collection("todos")
            .insert_one(doc! {
                "title": "Milk",
                "description": "Pick up from walmart.",
                "completed": false
            })
            .await
            .unwrap();

        let filter = doc! {"title": "General"};
        let update = doc! {
            "$push": {
                "todos": &todo_id.inserted_id,
            }
        };
        let insert = mongo
            .db
            .collection::<Document>("todo_lists")
            .update_one(filter, update)
            .await;

        let todo = mongo.get_todos("General").await.unwrap();
        println!("{:?}", todo);
    }
}
