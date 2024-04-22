use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct License {
    pub unique_id: String,
    pub user_id: String,
    pub license: String,
}