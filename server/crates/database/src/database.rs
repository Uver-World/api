use std::sync::Arc;

use mongodb::{error::Error, *};

use crate::managers::{OrganizationManager, PeersManager, UserManager};

#[derive(Clone)]
pub struct DatabaseSettings {
    // the database hostname
    pub hostname: String,
    // the database port
    pub port: u16,
    // The database username
    pub username: String,
    // The database password
    pub password: String,
    // The database name
    pub database: String,
}

#[derive(Clone)]
pub struct Database {
    pub user_manager: Arc<UserManager>,
    pub organization_manager: Arc<OrganizationManager>,
    pub peers_manager: Arc<PeersManager>,
}

impl Database {
    pub async fn init(settings: &DatabaseSettings) -> Result<Self, Error> {
        let uri = format!(
            "mongodb://{username}:{password}@{hostname}:{port}/",
            username = settings.username,
            password = settings.password,
            hostname = settings.hostname,
            port = settings.port
        );
        let client = Client::with_uri_str(uri).await?;
        let db = client.database(&settings.database);
        let names = db.list_collection_names(None).await?;
        if !names.contains(&"users".to_string()) {
            db.create_collection("users", None).await?;
        }
        Ok(Database {
            user_manager: Arc::new(UserManager::init(db.collection("users"))),
            organization_manager: Arc::new(OrganizationManager::init(db.collection("organizations"))),
            peers_manager: Arc::new(PeersManager::init(db.collection("peers"))),
        })
    }
}
