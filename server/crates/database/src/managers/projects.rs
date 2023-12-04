pub use crate::models::project::Project;

use futures::StreamExt;
use mongodb::{
    error::Error,
    results::InsertOneResult,
    bson::doc,
    Collection,
};

pub struct ProjectManager {
    pub projects: Collection<Project>,
}

impl ProjectManager {
    pub fn init(projects: Collection<Project>) -> Self {
        Self { projects }
    }

    pub async fn create(
        &self,
        project: &Project,
    ) -> Result<InsertOneResult, Error> {
        let target = self.projects.insert_one(project, None).await?;
        Ok(target)
    }

    pub async fn from_organization_id(
        &self,
        organization_id: &str,
    ) -> Result<Vec<Project>, Error> {
        let filter = doc! {"organization_id": organization_id};
        let mut cursor: mongodb::Cursor<Project>  = self.projects.find(filter, None).await?;
        let mut projects = Vec::new();
        
        while let Some(project) = cursor.next().await {
            projects.push(project?);
        }

        Ok(projects)
    }

    pub async fn from_id(
        &self,
        id: &str,
    ) -> Result<Option<Project>, Error> {
        Ok(self.projects.find_one(doc! {"unique_id": id}, None).await?)
    }

    pub async fn delete_from_id(
        &self,
        id: &str
    ) -> Result<Option<Project>, Error> {
        Ok(self.projects.find_one_and_delete(doc! {"unique_id": id}, None).await?)
    }
}

