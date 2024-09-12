use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, JsonSchema, Clone)]
pub struct Comment {
    pub user_id: u32,
    pub content: String,
    pub timestamp: String,
}

impl Comment {}
