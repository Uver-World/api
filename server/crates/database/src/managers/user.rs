use std::collections::HashMap;

use mongodb::{
    bson::{doc, to_bson, Bson},
    error::Error,
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Collection,
};

use crate::{authentication::Authentication, models::user::User, user::UserUpdate, group::Group};

pub struct UserManager {
    pub users: Collection<User>,
}

impl UserManager {
    pub fn init(users: Collection<User>) -> Self {
        Self { users }
    }

    pub async fn email_exists(&self, email: String) -> Result<bool, Error> {
        Ok(self
            .users
            .count_documents(doc! { "authentication.Credentials.email": email }, None)
            .await?
            != 0)
            
    }

    pub async fn create_user(&self, user: &User) -> Result<InsertOneResult, Error> {
        let target = self.users.insert_one(user, None).await?;
        Ok(target)
    }

    pub async fn from_token(&self, token: &str) -> Result<Option<User>, Error> {
        match self
            .users
            .find_one(doc! { "logins.token": token}, None)
            .await?
        {
            Some(user) => Ok(Some(user)),
            None => Ok(None),
        }
    }

    pub async fn from_id(&self, id: &str) -> Result<Option<User>, Error> {
        match self
            .users
            .find_one(doc! { "unique_id": id.to_string() }, None)
            .await?
        {
            Some(user) => Ok(Some(user)),
            None => Ok(None),
        }
    }

    pub async fn from_email(&self, email: &str) -> Result<Option<User>, Error> {
        match self
            .users
            .find_one(doc! { "authentication.Credentials.email": email.to_string() }, None)
            .await?
        {
            Some(user) => Ok(Some(user)),
            None => Ok(None),
        }
    }

    pub async fn delete_user(
        &self,
        uuid: Option<&str>,
        token: Option<&str>,
    ) -> Result<Option<DeleteResult>, String> {
        let user = if let Some(uuid) = uuid {
            self.from_id(uuid)
                .await
                .map_err(|err| err.to_string())?
                .ok_or(format!("User not found with id: {uuid}"))?
        } else if let Some(token) = token {
            self.from_token(token)
                .await
                .map_err(|err| err.to_string())?
                .ok_or(format!("User not found with token: {token}"))?
        } else {
            return Ok(None);
        };
        Ok(Some(
            self.users
                .delete_one(doc! {"unique_id": user.unique_id}, None)
                .await
                .map_err(|err| err.to_string())?,
        ))
    }

    pub async fn update_auth(
        &self,
        uuid: String,
        new_auth: &Authentication,
    ) -> Result<UpdateResult, Error> {
        let filter = doc! {"unique_id": uuid.to_string()};
        let update = doc! {"$set": {"authentication": to_bson(new_auth).unwrap()}};
        self.users.update_one(filter, update, None).await
    }

    pub async fn update_user(
        &self,
        uuid: String,
        user_update: Vec<UserUpdate>,
    ) -> Result<UpdateResult, Error> {
        let filter = doc! {"unique_id": uuid.to_string()};
        let fields: HashMap<String, Bson> = user_update
            .iter()
            .filter_map(|update| update.convert())
            .collect();
        let update = doc! {"$set": to_bson(&fields).unwrap()};
        self.users.update_one(filter, update, None).await
    }

    pub async fn website_missing(&self) -> Result<bool, Error> {
        let filter = doc! {"group": format!("{:?}", Group::Website)};
        let documents = self.users.count_documents(filter, None).await?;
        Ok(documents == 0)
    }
}

impl Clone for UserManager {
    fn clone(&self) -> Self {
        Self {
            users: self.users.clone(),
        }
    }
}