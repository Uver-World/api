use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, JsonSchema, Serialize)]
pub struct ProjectInit {
    pub name: String
}

impl ProjectInit {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
