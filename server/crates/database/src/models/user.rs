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
            Self::Username(username) => to_bson(username)
                .map(|username| ("username".to_string(), username))
                .ok(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, Clone)]
pub struct User {
    pub authentication: Authentication,
    pub unique_id: String,
    pub creation_date: String,
    pub logins: Vec<Login>,
    pub username: String,
    pub group: Group,
}

impl User {
    pub async fn upload_token(&self, login: &Login, users: &Collection<User>) {
        let _ = users
            .update_one(
                doc! {"unique_id": &self.unique_id },
                doc! {"$push": {"logins": to_bson(&login).unwrap()}},
                None,
            )
            .await;
    }
}
