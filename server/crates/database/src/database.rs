use bson::doc;
use mongodb::{error::Error, *};

use crate::{managers::{LicenseManager, OrganizationManager, PeersManager, PermissionManager, ProjectManager, UserManager, AssetManager}, permission::Permission};

use crate::server::Server;

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
    pub user_manager: UserManager,
    pub organization_manager: OrganizationManager,
    pub peers_manager: PeersManager,
    pub project_manager: ProjectManager,
    pub license_manager: LicenseManager,
    pub permission_manager: PermissionManager,
    pub asset_manager: AssetManager,
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
        if !names.contains(&"licenses".to_string()) {
            db.create_collection("licenses", None).await?;
        }
        if !names.contains(&"permissions".to_string()) {
            db.create_collection("permissions", None).await?;
        }
        if !names.contains(&"assets".to_string()) {
            db.create_collection("assets", None).await?;
        }
        if !names.contains(&"comments".to_string()) {
            db.create_collection("comments", None).await?;
        }

        // clear permissions collection
        let permissions = vec![
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "organisation.all".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "organisation.see".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "organisation.edit".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "organisation.create".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "organisation.members.all".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "organisation.members.add".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "organisation.members.remove".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "organisation.members.edit".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "organisation.events.see".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "profile.edit".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "profile.see".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "profile.reset_password".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "license.create".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "client.download".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "project.see".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "project.edit".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "project.create".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "permission.add".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "permission.remove".to_string(),
            },
            Permission {
                unique_id: Server::generate_unique_id().to_string(),
                name: "permission.see".to_string(),
            },
        ];
        db.collection::<Permission>("permissions").delete_many(doc! {}, None).await?;
        db.collection::<Permission>("permissions").insert_many(permissions, None).await?;

        Ok(Database {
            user_manager: UserManager::init(db.collection("users")),
            organization_manager: OrganizationManager::init(db.collection("organizations")),
            peers_manager: PeersManager::init(db.collection("peers")),
            project_manager: ProjectManager::init(db.collection("projects")),
            license_manager: LicenseManager::init(db.collection("licenses")),
            permission_manager: PermissionManager::init(db.collection("permissions")),
            asset_manager: AssetManager::init(db.collection("assets")),
        })
    }
}