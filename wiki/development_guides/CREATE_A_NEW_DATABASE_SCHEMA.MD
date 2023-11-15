[return to list of guides](../README.MD)
# Create a new database schema
> If you want to implement a new database schema on the rest api, you can do it by following these steps:

**1. Create a new manager to manage your new database schema in: `server/crates/database/src/managers/name_of_database_schema.rs`**

**2. Then add a test Collection to test if it works**
```rust
use mongodb::{
    bson::doc,
    error::Error,
    results::{DeleteResult, InsertOneResult},
    Collection,
};

pub struct UsernamesManager {
    pub usernames: Collection<String>,
}

impl UsernamesManager {
    pub fn init(usernames: Collection<String>) -> Self {
        Self { usernames }
    }

    pub async fn username_exists(&self, username: impl Into<String>) -> Result<bool, Error> {
        Ok(self
            .usernames
            .count_documents(doc! { "username": username.into() }, None)
            .await?
            != 0)
    }

    pub async fn create_username(&self, username: String) -> Result<InsertOneResult, Error> {
        let target = self.usernames.insert_one(username, None).await?;
        Ok(target)
    }

    pub async fn delete_username(
        &self,
        username: &str,
    ) -> Result<Option<DeleteResult>, String> {
        Ok(Some(
            self.usernames
                .delete_one(doc! {"username": username}, None)
                .await
                .map_err(|err| err.to_string())?,
        ))
    }
}

```

**3. Register your database schema at: `server/crates/database/src/database.rs`**
```rust
pub struct Database {
    // ...,
    pub usernames_manager: UsernamesManager,
}

impl Database {
    pub async fn init(settings: &DatabaseSettings) -> Result<Self, Error> {
        // ...,
        Ok(Database {
            // ...,
            usernames_manager: UsernamesManager::init(db.collection("usernames")),
        })
    }
}
```


```rs
You're done, you can now use your new database schema
```

[return to list of guides](../README.MD)