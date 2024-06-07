use mongodb::{
    bson::{doc, to_bson, Bson},
    Collection,
};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{authentication::Authentication, group::Group, login::Login};

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub enum UserUpdate {
    Username(String),
}

impl UserUpdate {
    pub fn convert(&self) -> Option<(String, Bson)> {
        match self {
            UserUpdate::Username(username) => Some((
                "authentication.Credentials.username".to_string(),
                to_bson(username).unwrap(),
            )),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, Clone)]
pub struct User {
    pub authentication: Authentication,
    pub unique_id: String,
    pub creation_date: String,
    pub logins: Vec<Login>,
    pub group: Group,
    pub permissions: Vec<String>,
}

impl User {
    /// Returns the last token
    pub fn get_token(&self) -> Option<&String> {
        Some(&self.logins.last()?.token.0)
    }

    pub async fn upload_token(&self, login: &Login, users: &Collection<Self>) {
        let _ = users
            .update_one(
                doc! {"unique_id": &self.unique_id },
                doc! {"$push": {"logins": to_bson(&login).unwrap()}},
                None,
            )
            .await;
    }

    pub fn default_website_user(unique_id: String, timestamp: u128) -> Self {
        Self {
            authentication: Authentication::None,
            unique_id,
            creation_date: timestamp.to_string(),
            logins: vec![Login::new("127.0.0.1".to_string(), timestamp, Authentication::None)],
            group: Group::Website,
            permissions: vec![],
        }
    }
}
