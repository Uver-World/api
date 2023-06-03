use mongodb::{
    bson::{doc, to_bson},
    options::FindOneOptions,
    Collection,
};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{group::Group, user::User};

#[derive(Serialize, Debug, Deserialize, Clone, JsonSchema, PartialEq)]
pub enum Authentication {
    Credentials(Credentials),
    None,
}

#[derive(Deserialize, Debug, Serialize, Clone, JsonSchema, PartialEq)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

impl Credentials {
    pub fn new_auth(self) -> Authentication {
        Authentication::Credentials(self)
    }
}

impl Authentication {
    pub async fn register(
        &self,
        timestamp: u128,
        unique_id: String,
        users: &Collection<User>,
    ) -> Result<Option<User>, String> {
        if !matches!(self, Authentication::None) {
            let existing_user = self.get(users).await;
            if existing_user.is_ok() && existing_user.unwrap().is_some() {
                return Ok(None);
            }
        }
        let user = User {
            authentication: self.clone(),
            unique_id: unique_id.clone(),
            creation_date: timestamp.to_string(),
            logins: Vec::new(),
            username: unique_id.clone(),
            group: Group::Guest,
        };

        let _ = users
            .insert_one(&user, None)
            .await
            .map_err(|err| err.to_string())?;
        Ok(Some(user))
    }

    pub async fn get(&self, users: &Collection<User>) -> Result<Option<User>, String> {
        users
            .find_one(
                doc! {"authentication": to_bson(&self).unwrap()},
                Some(
                    FindOneOptions::builder()
                        .projection(doc! {"token": 0})
                        .build(),
                ),
            )
            .await
            .map_err(|err| err.to_string())
    }

    pub fn get_name(&self) -> String {
        match self {
            Self::Credentials(_) => "Credentials",
            Self::None => "None",
        }
        .to_owned()
    }

    pub fn token(&self) -> Option<String> {
        None
    }
}
