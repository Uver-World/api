use crate::models::permission::Permission;

use mongodb::{
  error::Error, results::InsertOneResult, Collection
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
}

impl Clone for PermissionManager {
  fn clone(&self) -> Self {
    Self {
      permissions: self.permissions.clone(),
    }
  }
}
