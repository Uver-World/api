use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
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
