use crate::models::permission::Permission;

use mongodb::{
  bson::doc, error::Error, results::InsertOneResult, Collection
};

pub struct PermissionManager {
  pub permissions: Collection<Permission>,
}

impl PermissionManager {
  pub fn init(permissions: Collection<Permission>) -> Self {
    Self { permissions }
  }

  pub async fn create(&self, permission: &Permission) -> Result<InsertOneResult, Error> {
    let target = self.permissions.insert_one(permission, None).await?;
    Ok(target)
  }

  pub async fn permission_exists(&self, permission_id: &str) -> bool {
    let filter = doc! { "unique_id": permission_id };
    let result = self.permissions.find_one(filter, None).await;
    match result {
      Ok(permission) => match permission {
        Some(_) => true,
        None => false,
      },
      Err(_) => false,
    }
  }

  pub async fn get_permission_id(&self, permission_name: &str) -> Result<String, Error> {
    let filter = doc! { "name": permission_name };
    let result = self.permissions.find_one(filter, None).await;
    match result {
      Ok(permission) => match permission {
        Some(permission) => Ok(permission.unique_id),
        None => Err(Error::from(std::io::Error::new(
          std::io::ErrorKind::NotFound,
          "Unknown permission",
        ))),
      },
      Err(err) => Err(err),
    }
  }
}

impl Clone for PermissionManager {
  fn clone(&self) -> Self {
    Self {
      permissions: self.permissions.clone(),
    }
  }
}
