use dotenv::dotenv;
use mongodb::{
    Client, Database,
    bson::{Document, doc},
    error,
};
use std::env;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::OnceCell;

    static MONGO_INI: OnceCell<MongoCrud> = OnceCell::const_new();

    async fn get_test_mongo() -> &'static MongoCrud {
        MONGO_INI
            .get_or_init(|| async {
                MongoCrud::connect()
                    .await
                    .expect("Failed to connect to MongoDB Atlas")
            })
            .await
    }

    #[tokio::test]
    async fn test_connect() {
        let mongo = get_test_mongo();
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
}
