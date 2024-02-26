pub use crate::models::license::License;

use mongodb::{
    error::Error,
    results::InsertOneResult,
    Collection,
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
}

