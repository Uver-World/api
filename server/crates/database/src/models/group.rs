use std::fmt;

use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, JsonSchema, Clone, PartialEq, Copy)]
pub enum Group {
    /// Guest has access to all public routes, might be an unknown user, or an user that didn't buy the licence
    Guest,
    /// User has access to authenticated routes
    User,
    /// Server has access to server routes
    Server,
    /// Website has access to website routes, basically has full perms on the API.
    Website,
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Guest => write!(f, "Guest"),
            Self::User => write!(f, "User"),
            Self::Server => write!(f, "Server"),
            Self::Website => write!(f, "Website"),
        }
    }
}
