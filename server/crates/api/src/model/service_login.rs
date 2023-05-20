use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, Debug, JsonSchema)]
pub struct Settings {
    pub token: String,
    pub uri: Option<String>,
}

#[derive(Deserialize, Debug, JsonSchema)]
pub enum ServiceLogin {
    Discord(Settings),
    Github(Settings),
    Mastodon(Settings),
}

impl ServiceLogin {
    pub fn name(&self) -> String {
        match self {
            Self::Mastodon(_) => "Mastodon".to_owned(),
            Self::Discord(_) => "Discord".to_owned(),
            Self::Github(_) => "Github".to_owned(),
        }
    }
}
