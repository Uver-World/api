use std::net::SocketAddr;

use database::{
    authentication::Authentication,
    group::Group,
    user::{User, UserUpdate},
    Database,
};
use rocket::{http::Status, response::status::Custom, serde::json::Json, *};
use rocket_okapi::openapi;

use crate::model::{login::Login, user_token::UserData};

mod helper;

/// Retrieve the user informations from its token
#[openapi(tag = "Users")]
#[get("/token/<token>")] // <- route attribute
pub async fn from_token(
    user_data: UserData,
    database: &State<Database>,
    token: String,
) -> Custom<Result<Json<User>, String>> {
    if matches!(user_data.group, Group::Website | Group::Server) == false {
        return Custom(
            Status::Forbidden,
            Err("Website or Server group required.".into()),
        );
    }
    match database.user_manager.from_token(&token).await {
        Ok(user) if user.is_some() => Custom(Status::Ok, Ok(Json(user.unwrap()))),
        _ => Custom(
            Status::NotFound,
            Err(format!("User not found with token: {token}")),
        ),
    }
}

/// Retrieve the user informations from its unique identifier
#[openapi(tag = "Users")]
#[get("/id/<id>")] // <- route attribute
pub async fn from_id(
    user_data: UserData,
    database: &State<Database>,
    id: u64,
) -> Custom<Result<Json<User>, String>> {
    if matches!(user_data.group, Group::Website | Group::Server) == false {
        return Custom(
            Status::Forbidden,
            Err("Website or Server group required.".into()),
        );
    }
    match database.user_manager.from_id(&id.to_string()).await {
        Ok(user) if user.is_some() => Custom(Status::Ok, Ok(Json(user.unwrap()))),
        _ => Custom(
            Status::NotFound,
            Err(format!("User not found with id: {id}")),
        ),
    }
}

/// Update the user informations from its token
#[openapi(tag = "Users")]
#[patch("/token/<token>", data = "<user_update>", format = "application/json")] // <- route attribute
pub async fn update(
    user_data: UserData,
    database: &State<Database>,
    token: String,
    user_update: Json<Vec<UserUpdate>>,
) -> Custom<Result<Json<bool>, String>> {
    if matches!(user_data.group, Group::Website) == false {
        return Custom(Status::Forbidden, Err("Website group required.".into()));
    }
    match database.user_manager.from_token(&token).await {
        Ok(user) if user.is_some() => {
            let uuid = user.unwrap().unique_id;
            match database.user_manager.update_user(uuid, user_update.0).await {
                Ok(_) => Custom(Status::Ok, Ok(Json(true))),
                Err(err) => Custom(Status::InternalServerError, Err(err.to_string())),
            }
        }

        _ => Custom(
            Status::InternalServerError,
            Err(format!("User not found with token: {token}")),
        ),
    }
}

/// Delete the user linked to the token
#[openapi(tag = "Users")]
#[delete("/token/<token>")] // <- route attribute
pub async fn delete_from_token(
    user_data: UserData,
    database: &State<Database>,
    token: String,
) -> Custom<Result<Json<bool>, String>> {
    if matches!(user_data.group, Group::Website) == false {
        return Custom(Status::Forbidden, Err("Website group required.".into()));
    }
    match database.user_manager.delete_user(None, Some(&token)).await {
        Ok(_) => Custom(Status::Ok, Ok(Json(true))),
        Err(err) => Custom(Status::InternalServerError, Err(err)),
    }
}

/// Delete the user from its id
#[openapi(tag = "Users")]
#[delete("/id/<id>")] // <- route attribute
pub async fn delete_from_id(
    user_data: UserData,
    database: &State<Database>,
    id: String,
) -> Custom<Result<Json<bool>, String>> {
    if matches!(user_data.group, Group::Website) == false {
        return Custom(Status::Forbidden, Err("Website group required.".into()));
    }
    match database.user_manager.delete_user(Some(&id), None).await {
        Ok(_) => Custom(Status::Ok, Ok(Json(true))),
        Err(err) => Custom(Status::InternalServerError, Err(err)),
    }
}

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
            if matches!(user_data.group, Group::Website) == false {
                return Custom(Status::Forbidden, "Website group required.".into());
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
    if matches!(user_data.group, Group::Website) == false {
        return Custom(Status::Forbidden, "Website group required.".into());
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
