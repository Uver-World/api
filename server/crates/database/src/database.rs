use std::sync::Arc;

use mongodb::{error::Error, *};

use crate::managers::{OrganizationManager, PeersManager, UserManager, ProjectManager};

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
<<<<<<< HEAD
    pub user_manager: Arc<UserManager>,
    pub organization_manager: Arc<OrganizationManager>,
    pub peers_manager: Arc<PeersManager>,
=======
    pub user_manager: UserManager,
    pub organization_manager: OrganizationManager,
    pub peers_manager: PeersManager,
    pub project_manager: ProjectManager,
>>>>>>> 199798a (UVW-3 Add create project route)
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
        if !names.contains(&"organizations".to_string()) {
            db.create_collection("organizations", None).await?;
        }
        if !names.contains(&"projects".to_string()) {
            db.create_collection("projects", None).await?;
        }
        Ok(Database {
<<<<<<< HEAD
            user_manager: Arc::new(UserManager::init(db.collection("users"))),
            organization_manager: Arc::new(OrganizationManager::init(db.collection("organizations"))),
            peers_manager: Arc::new(PeersManager::init(db.collection("peers"))),
=======
            user_manager: UserManager::init(db.collection("users")),
            organization_manager: OrganizationManager::init(db.collection("organizations")),
            peers_manager: PeersManager::init(db.collection("peers")),
            project_manager: ProjectManager::init(db.collection("projects")),
>>>>>>> 199798a (UVW-3 Add create project route)
        })
    }
}
