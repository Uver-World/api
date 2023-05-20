use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{authentication::Authentication, token::Token};

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct Login {
    ip: String,
    timestamp: String,
    method: String,
    pub token: Token,
}

impl Login {
    pub fn new(ip: String, timestamp: u128, method: Authentication) -> Self {
        let token = Token::new();

        Self {
            ip,
            timestamp: timestamp.to_string(),
            method: method.get_name(),
            token,
        }
    }
}
