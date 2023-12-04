pub use crate::models::project::Project;

use mongodb::{
    error::Error,
    results::{InsertOneResult},
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
}

