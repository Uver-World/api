use database::group::Group;
use database::Database;

use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::response::status::Custom;
use rocket_okapi::okapi::openapi3::{
    Object, SecurityRequirement, SecurityScheme, SecuritySchemeData,
};
use rocket_okapi::{
    gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
};

use serde::Deserialize;
#[derive(Deserialize)]
pub struct UserData {
    pub id: Option<String>,
    pub group: Group,
}

impl UserData {
    pub fn matches_group(&self, groups: Vec<Group>) -> Result<(), Custom<String>> {
        if matches!(self.group, Group::Guest) {
            return Err(Custom(Status::Forbidden, "Authentication required.".into()));
        }
        if !groups.contains(&self.group) {
            return Err(Custom(
                Status::Unauthorized,
                format!("You need to be part of one of the following groups: [{groups:?}]."),
            ));
        }
        Ok(())
    }
}

impl UserData {
    fn new(id: Option<String>, group: Group) -> Self {
        Self { id, group }
    }
}

#[derive(Debug)]
pub enum UserDataError {
    BadCount,
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserData {
    type Error = UserDataError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<UserData, Self::Error> {
        let keys: Vec<_> = request.headers().get("X-User-Token").collect();
        match keys.len() {
            0 => return Outcome::Success(UserData::new(None, Group::Guest)),
            1 => {
                let token = keys.first().unwrap();

                let db = request.rocket().state::<Database>().unwrap();

                let user = db.user_manager.from_token(token).await;

                if user.is_ok() && let Some(user) = user.as_ref().unwrap() {
                    let user = user.clone();
                    return Outcome::Success(UserData::new(Some(user.unique_id), user.group));
                }
                return Outcome::Success(UserData::new(None, Group::Guest));
            }
            _ => {
                return Outcome::Success(UserData::new(None, Group::Guest));
            }
        }
    }
}

impl<'a> OpenApiFromRequest<'a> for UserData {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        let security_scheme = SecurityScheme {
            description: Some("Requires a User token to access".to_string()),
            data: SecuritySchemeData::ApiKey {
                name: "X-User-Token".to_owned(),
                location: "header".to_owned(),
            },
            extensions: Object::default(),
        };
        // Add the requirement for this route/endpoint
        // This can change between routes.
        let mut security_req = SecurityRequirement::new();
        // Each security requirement needs to be met before access is allowed.
        security_req.insert("UserTokenAuth".to_owned(), Vec::new());
        // These vvvvvvv-----^^^^^^^^^^ values need to match exactly!
        Ok(RequestHeaderInput::Security(
            "UserTokenAuth".to_owned(),
            security_scheme,
            security_req,
        ))
    }
}
