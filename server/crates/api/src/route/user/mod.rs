use database::Database;
use rocket::{http::Status, response::status::Custom, serde::json::Json, *};
use rocket_okapi::openapi;

mod helper;

mod route_delete_from_id;
mod route_delete_from_token;
mod route_from_id;
mod route_from_token;
mod route_register;
mod route_renew;
mod route_update;
mod route_update_auth;

pub use route_delete_from_id::*;
pub use route_delete_from_token::*;
pub use route_from_id::*;
pub use route_from_token::*;
pub use route_register::*;
pub use route_renew::*;
pub use route_update::*;
pub use route_update_auth::*;

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
