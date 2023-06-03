use std::collections::HashMap;

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

    pub async fn uuid_exists(&self, uuid: String) -> Result<bool, Error> {
        Ok(self
            .organizations
            .count_documents(doc! { "unique_id": uuid }, None)
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

    pub async fn update_organization(
        &self,
        uuid: String,
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
}
