use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, JsonSchema, Serialize)]
pub struct OrganizationServer {
    pub organization_id: String,
    pub server_id: String,
}
