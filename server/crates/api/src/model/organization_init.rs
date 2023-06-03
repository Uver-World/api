use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, JsonSchema, Serialize)]
pub struct OrganizationInit {
    pub name: String,
    pub owner_id: String,
}

impl OrganizationInit {
    pub fn new(name: String, owner_id: String) -> Self {
        Self { name, owner_id }
    }
}
