use database::authentication::Credentials;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, Debug, JsonSchema)]
pub enum Login {
    Credentials(Credentials),
    UserId(String),
}

impl Login {
    pub fn name(&self) -> String {
        match self {
            Self::Credentials(_) => "Credentials".to_owned(),
            Self::UserId(_) => "UserId".to_owned(),
        }
    }
}
