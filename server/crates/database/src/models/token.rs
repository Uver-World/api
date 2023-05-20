use rand::{distributions::Alphanumeric, Rng};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, JsonSchema, Debug)]
pub struct Token(pub String);

impl Token {
    pub fn new() -> Self {
        let raw_token = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        Self(raw_token)
    }
}
