use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, JsonSchema, Serialize)]
pub struct Project {
    pub unique_id: String,
    pub organization_id: String,
    pub member_ids: Vec<String>,
    pub name: String
}
