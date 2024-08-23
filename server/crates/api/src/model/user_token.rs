use database::Database;

use rocket::request::{self, FromRequest, Outcome, Request};
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
}

impl UserData {
    fn new(id: Option<String>) -> Self {
        Self { id }
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
            0 => return Outcome::Success(UserData::new(None)),
            1 => {
                let token = keys.first().unwrap();

                let db = request.rocket().state::<Database>().unwrap();

                let user = db.user_manager.from_token(token).await;

                if user.is_ok() && let Some(user) = user.as_ref().unwrap() {
                    let user = user.clone();
                    return Outcome::Success(UserData::new(Some(user.unique_id)));
                }
                return Outcome::Success(UserData::new(None));
            }
            _ => {
                return Outcome::Success(UserData::new(None));
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
