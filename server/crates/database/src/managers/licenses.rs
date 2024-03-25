pub use crate::models::license::License;

use futures::StreamExt;
use mongodb::{
    bson::doc, error::Error, results::InsertOneResult, Collection
};

pub struct LicenseManager {
    pub licenses: Collection<License>,
}

impl LicenseManager {
    pub fn init(licenses: Collection<License>) -> Self {
        Self { licenses }
    }

    pub async fn create(
        &self,
        license: &License,
    ) -> Result<InsertOneResult, Error> {
        let target = self.licenses.insert_one(license, None).await?;
        Ok(target)
    }

    pub async fn get_licenses(
        &self,
        user_id: &str,
    ) -> Result<Vec<License>, Error> {
        let filter = doc! {"user_id": user_id};
        let mut cursor: mongodb::Cursor<License>  = self.licenses.find(filter, None).await?;
        let mut licenses = Vec::new();
        
        while let Some(project) = cursor.next().await {
            licenses.push(project?);
        }

        Ok(licenses)
    }

    pub async fn get_license(
        &self,
        license_id: &str,
    ) -> Result<Option<License>, Error> {
        match self
            .licenses
            .find_one(doc! { "license": license_id.to_string() }, None)
            .await?
        {
            Some(license) => Ok(Some(license)),
            None => Ok(None),
        }
    }
}