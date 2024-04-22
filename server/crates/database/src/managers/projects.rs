pub use crate::models::project::Project;
pub use crate::models::project::ProjectUpdateData;
pub use crate::models::project::ProjectUpdate;
use std::collections::HashMap;


use futures::StreamExt;
use mongodb::{
    error::Error,
    results::InsertOneResult,
    bson::{doc, to_bson, Bson},
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

    pub async fn update_project(
        &self,
        id: &str,
        project_update: Vec<ProjectUpdate>,
    ) -> Result<bool, Error> {
        let filter = doc! {"unique_id": id.to_string()};
        let fields: HashMap<String, Bson> = project_update
            .iter()
            .filter_map(|update| update.convert())
            .collect();
        let update = doc! {"$set": to_bson(&fields).unwrap()};

        match self.projects.update_one(filter, update, None).await {
            Ok(result) if result.matched_count > 0 => Ok(true),
            Ok(_) => Ok(false),
            Err(err) => Err(err),
        }
    }
}

impl Clone for ProjectManager {
    fn clone(&self) -> Self {
        Self {
            projects: self.projects.clone(),
        }
    }
}