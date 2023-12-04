use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct Project {
    pub unique_id: String,
    pub organization_id: String,
    pub name: String,
    pub member_ids: Vec<String>,
}