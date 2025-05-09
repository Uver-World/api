use mongodb::{
    bson::doc,
    options::FindOneOptions,
    Collection,
};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{user::User};

#[derive(Serialize, Debug, Deserialize, Clone, JsonSchema, PartialEq)]
pub enum Authentication {
    Credentials(Credentials),
    None,
}

#[derive(Deserialize, Debug, Serialize, Clone, JsonSchema, PartialEq)]
pub struct Credentials {
    pub email: String,
    pub username: Option<String>,
    pub avatar: Option<String>,
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

        // Store the avatar in ./uploads/avatars
        // Example:
        let avatar_filename = match &self {
            Authentication::Credentials(credentials) => {
                if let Some(avatar) = &credentials.avatar {
                    avatar
                } else {
                    "default_avatar.jpg" // Or any default avatar filename
                }
            }
            Authentication::None => "default_avatar.jpg",
        };
        let avatar_path = format!("./uploads/avatars/{}", avatar_filename);
        let mut self_clone = self.clone();
        if let Authentication::Credentials(credentials) = &mut self_clone {
            credentials.avatar = Some(avatar_path.clone());
        }

        let user = User {
            authentication: self.clone(),
            unique_id: unique_id.clone(),
            creation_date: timestamp.to_string(),
            logins: Vec::new(),
            permissions: Vec::new(),
        };


        let _ = users
            .insert_one(&user, None)
            .await
            .map_err(|err| err.to_string())?;
        Ok(Some(user))
    }

    pub async fn get(&self, users: &Collection<User>) -> Result<Option<User>, String> {
        match self {
            Authentication::Credentials(credentials) => {
                let filter = doc! {"authentication.Credentials.email": &credentials.email, "authentication.Credentials.password": &credentials.password};
                let options = FindOneOptions::default();

                if let Some(user) = users.find_one(filter, options).await.map_err(|err| err.to_string())? {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            Authentication::None => Ok(None),
        }
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

    pub fn credentials(&self) -> &Credentials {
        match self {
            Self::Credentials(credentials) => credentials,
            Self::None => panic!("No credentials"),
        }
    }
}