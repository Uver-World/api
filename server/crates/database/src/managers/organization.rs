use std::collections::HashMap;

use futures::StreamExt;
use mongodb::{
    bson::{doc, to_bson, Bson},
    error::Error,
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Collection,
};

use crate::organization::{Organization, OrganizationUpdate};

pub struct OrganizationManager {
    pub organizations: Collection<Organization>,
}

impl OrganizationManager {
    pub fn init(organizations: Collection<Organization>) -> Self {
        Self { organizations }
    }

    pub async fn organization_exists(&self, name: String) -> Result<bool, Error> {
        Ok(self
            .organizations
            .count_documents(doc! { "name": name }, None)
            .await?
            != 0)
    }

    pub async fn create_organization(
        &self,
        organization: &Organization,
    ) -> Result<InsertOneResult, Error> {
        let target = self.organizations.insert_one(organization, None).await?;
        Ok(target)
    }

    pub async fn from_id(&self, id: &str) -> Result<Option<Organization>, Error> {
        match self
            .organizations
            .find_one(doc! { "unique_id": id.to_string() }, None)
            .await?
        {
            Some(organization) => Ok(Some(organization)),
            None => Ok(None),
        }
    }

    pub async fn delete_organization(&self, uuid: &str) -> Result<Option<DeleteResult>, String> {
        Ok(Some(
            self.organizations
                .delete_one(doc! {"unique_id": uuid}, None)
                .await
                .map_err(|err| err.to_string())?,
        ))
    }

    pub async fn add_to_server_ids(
        &self,
        uuid: &str,
        server_id: &str,
    ) -> Result<UpdateResult, String> {
        let filter = doc! { "unique_id": uuid };
        let update = doc! { "$addToSet": { "server_ids": server_id } };

        self.organizations
            .update_one(filter, update, None)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn has_access_to_server(
        &self,
        server_unique_id: &str,
        user_unique_id: &str,
    ) -> Result<bool, String> {
        let filter = doc! {
            { "server_ids" }: { "$in": [server_unique_id] },
            "$or": [
                { "member_ids": { "$in": [user_unique_id] } },
                { "owner_id": user_unique_id }
            ]
        };
        match self
            .organizations
            .find_one(filter, None)
            .await
            .map_err(|err| err.to_string())?
        {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    pub async fn is_in_organization(
        &self,
        uuid: &str,
        user_unique_id: &str,
    ) -> Result<bool, String> {
        let filter = doc! {
            "unique_id": uuid,
            "$or": [
                { "member_ids": { "$in": [user_unique_id] } },
                { "owner_id": user_unique_id }
            ]
        };
        match self
            .organizations
            .find_one(filter, None)
            .await
            .map_err(|err| err.to_string())?
        {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    pub async fn remove_from_server_ids(
        &self,
        uuid: &str,
        server_id: &str,
    ) -> Result<UpdateResult, String> {
        let filter = doc! { "unique_id": uuid };
        let update = doc! { "$pull": { "server_ids": server_id } };

        self.organizations
            .update_one(filter, update, None)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn update_organization(
        &self,
        uuid: &str,
        organization_update: Vec<OrganizationUpdate>,
    ) -> Result<UpdateResult, Error> {
        let filter = doc! {"unique_id": uuid.to_string()};
        let fields: HashMap<String, Bson> = organization_update
            .iter()
            .filter_map(|update| update.convert())
            .collect();
        let update = doc! {"$set": to_bson(&fields).unwrap()};
        self.organizations.update_one(filter, update, None).await
    }

    // Find every organizations where owner_id = user_id or member_ids contains user_id
    pub async fn get_organizations_from_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<Organization>, Error> {
        let filter = doc! {
            "$or": [
                { "member_ids": { "$in": [user_id] } },
                { "owner_id": user_id }
            ]
        };
        let mut cursor: mongodb::Cursor<Organization> = self.organizations.find(filter, None).await?;
        let mut organizations: Vec<Organization> = Vec::new();

        while let Some(organization) = cursor.next().await {
            organizations.push(organization?);
        }

        Ok(organizations)
    }

    pub async fn add_member(&self, organization_id: &str, member_id: &str) -> Result<UpdateResult, Error> {
        let filter = doc! { "unique_id": organization_id };
        let update = doc! { "$addToSet": { "member_ids": member_id } };

        self.organizations
            .update_one(filter, update, None)
            .await
    }
}
