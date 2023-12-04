use mongodb::bson::{doc, to_bson, Bson};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub enum OrganizationUpdate {
    Name(String),
    OwnerId(String),
}

impl OrganizationUpdate {
    pub fn convert(&self) -> Option<(String, Bson)> {
        match self {
            Self::Name(name) => to_bson(name).map(|name| ("name".to_string(), name)).ok(),
            Self::OwnerId(name) => to_bson(name)
                .map(|name| ("owner_id".to_string(), name))
                .ok(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct Organization {
    pub unique_id: String,
    pub creation_date: String,
    pub name: String,
    pub member_ids: Vec<String>,
    pub owner_id: String,
    pub server_ids: Vec<String>,
    pub projects_ids: Vec<String>,
}