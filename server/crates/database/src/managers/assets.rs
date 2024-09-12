use mongodb::{
    bson::{doc, to_bson},
    error::Error,
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Collection,
};
use crate::models::asset::Asset;

pub struct AssetManager {
    pub assets: Collection<Asset>,
}

impl AssetManager {
    pub fn init(assets: Collection<Asset>) -> Self {
        Self { assets }
    }

    pub async fn create_asset(&self, asset: &Asset) -> Result<InsertOneResult, Error> {
        let result = self.assets.insert_one(asset, None).await?;
        Ok(result)
    }

    pub async fn get_asset_by_id(&self, id: u32) -> Result<Option<Asset>, Error> {
        match self.assets.find_one(doc! { "id": id }, None).await? {
            Some(asset) => Ok(Some(asset)),
            None => Ok(None),
        }
    }

    pub async fn update_asset(&self, id: u32, updated_asset: &Asset) -> Result<UpdateResult, Error> {
        let filter = doc! { "id": id };
        let update = doc! { "$set": to_bson(updated_asset).unwrap() };
        self.assets.update_one(filter, update, None).await
    }

    pub async fn delete_asset(&self, id: u32) -> Result<DeleteResult, Error> {
        let result = self.assets.delete_one(doc! { "id": id }, None).await?;
        Ok(result)
    }

    pub async fn asset_exists(&self, id: u32) -> Result<bool, Error> {
        let count = self.assets.count_documents(doc! { "id": id }, None).await?;
        Ok(count > 0)
    }
}

impl Clone for AssetManager {
    fn clone(&self) -> Self {
        Self {
            assets: self.assets.clone(),
        }
    }
}
