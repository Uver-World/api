use std::net::SocketAddr;

use database::{authentication::Authentication, group::Group, Database};
use rocket::{http::Status, response::status::Custom, serde::json::Json, *};
use rocket_okapi::openapi;

use crate::model::{login::Login, user_token::UserData};

mod helper;

mod route_delete_from_id;
mod route_delete_from_token;
mod route_from_id;
mod route_from_token;
mod route_update;

pub use route_delete_from_id::*;
pub use route_delete_from_token::*;
pub use route_from_id::*;
pub use route_from_token::*;
pub use route_update::*;

/// Renew a user token with either the user credentials, or with the serverid
///
/// To regenerate a server's token, you have to be part of the Website group
#[openapi(tag = "Users")]
#[post("/renew", data = "<login>", format = "application/json")] // <- route attribute
pub async fn renew(
    user_data: UserData,
    database: &State<Database>,
    login: Json<Login>,
    remot_addr: SocketAddr,
) -> Custom<String> {
    let ip = remot_addr.ip().to_string();
    match login.0 {
        Login::Credentials(credentials) => {
            let auth = Authentication::Credentials(credentials);
            let user = auth.get(&database.user_manager.users).await;
            helper::renew_token(user, ip, auth, &database.user_manager).await
        }
        Login::UserId(user_id) => {
            if let Err(response) = user_data.matches_group(vec![Group::Website]) {
                return Custom(response.0, response.1);
            }
            let user = database.user_manager.from_id(&user_id).await;
            helper::renew_token(
                user.map_err(|err| err.to_string()),
                ip,
                Authentication::None,
                &database.user_manager,
            )
            .await
        }
    }
}

/// Register a new user
///
/// If "Credentials" is being used, the user will have an authentication method of type credentials by default
///
/// Otherwise please don't use any
///
/// Requires 'Website' group
#[openapi(tag = "Users")]
#[post("/register", data = "<login>", format = "application/json")] // <- route attribute
pub async fn register(
    user_data: UserData,
    database: &State<Database>,
    login: Option<Json<Login>>,
    remot_addr: SocketAddr,
) -> Custom<String> {
    if let Err(response) = user_data.matches_group(vec![Group::Website]) {
        return Custom(response.0, response.1);
    }
    let ip = remot_addr.ip().to_string();
    if login.is_none() {
        return helper::register(Authentication::None, ip, &database.user_manager).await;
    }
    let login = login.unwrap();
    match login.0 {
        Login::Credentials(credentials) => {
            let auth = Authentication::Credentials(credentials);
            helper::register(auth, ip, &database.user_manager).await
        }
        _ => Custom(Status::BadRequest, "Credentials are required.".into()),
    }
}

/// Update the way an user authenticate itself
///
/// This update requires the user unique identifier
#[openapi(tag = "Users")]
#[patch("/id/<id>/update_auth", data = "<login>", format = "application/json")] // <- route attribute
pub async fn id_update_auth(
    database: &State<Database>,
    id: u64,
    login: Json<Login>,
) -> Custom<Result<Json<bool>, String>> {
    let id = match database.user_manager.from_id(&id.to_string()).await {
        Ok(user) if user.is_some() => user.unwrap().unique_id,
        _ => {
            return Custom(
                Status::NotFound,
                Err(format!("User not found with id: {id}")),
            );
        }
    };
    helper::update_auth(id, login, &database.user_manager).await
}

/// Update the way an user authenticates itself
///
/// This update requires the user token
#[openapi(tag = "Users")]
#[patch(
    "/token/<token>/update_auth",
    data = "<login>",
    format = "application/json"
)] // <- route attribute
pub async fn token_update_auth(
    database: &State<Database>,
    token: String,
    login: Json<Login>,
) -> Custom<Result<Json<bool>, String>> {
    let id = match database.user_manager.from_token(&token).await {
        Ok(user) if user.is_some() => user.unwrap().unique_id,
        _ => {
            return Custom(
                Status::NotFound,
                Err(format!("User not found with token: {token}")),
            );
        }
    };
    helper::update_auth(id, login, &database.user_manager).await
}

/// Check if an email is registered or not
#[openapi(tag = "Users")]
#[get("/email_exists/<email>")] // <- route attribute
pub async fn email_exists(
    database: &State<Database>,
    email: String,
) -> Custom<Result<Json<bool>, String>> {
    match database.user_manager.email_exists(email).await {
        Ok(value) => Custom(Status::Ok, Ok(Json(value))),
        _ => Custom(
            Status::InternalServerError,
            Err("Database error.".to_string()),
        ),
    }
}
