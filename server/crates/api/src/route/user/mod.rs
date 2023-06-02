use database::Database;
use rocket::{http::Status, response::status::Custom, serde::json::Json, *};
use rocket_okapi::openapi;

use crate::model::login::Login;

mod helper;

mod route_delete_from_id;
mod route_delete_from_token;
mod route_from_id;
mod route_from_token;
mod route_register;
mod route_renew;
mod route_update;

pub use route_delete_from_id::*;
pub use route_delete_from_token::*;
pub use route_from_id::*;
pub use route_from_token::*;
pub use route_register::*;
pub use route_renew::*;
pub use route_update::*;

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
