use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use mongodb::bson::{doc, to_bson, Bson};


#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct Project {
    pub unique_id: String,
    pub organization_id: String,
    pub name: String,
    pub member_ids: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct ProjectUpdateData {
    pub project_id: String,
    pub project_update: Vec<ProjectUpdate>,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub enum ProjectUpdate {
    Name(String),
}

impl ProjectUpdate {
    pub fn convert(&self) -> Option<(String, Bson)> {
        match self {
            Self::Name(name) => to_bson(name).map(|name| ("name".to_string(), name)).ok(),
        }
    }
}