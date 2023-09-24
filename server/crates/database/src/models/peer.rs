use mongodb::bson::doc;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct Peer {
    // The room_id is a 128 characters length string.
    pub room_id: String,
    pub creation_date: String,
    pub signaling_hostname: String,
    pub signaling_port: u16,
    pub server_unique_id: String,
}
