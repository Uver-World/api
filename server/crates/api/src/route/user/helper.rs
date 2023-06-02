use crate::{model::login::Login, RequestError, Server};
use database::{
    authentication::{Authentication, Credentials},
    managers::UserManager,
    user::User,
};

use rocket::{http::Status, response::status::Custom, serde::json::Json};

pub async fn renew_token(
    user: Result<Option<User>, String>,
    ip: String,
    auth: Authentication,
    usermanager: &UserManager,
) -> Custom<Result<String, Json<RequestError>>> {
    match user {
        Ok(user) if user.is_some() => {
            let login = database::login::Login::new(ip, Server::current_time(), auth);

            user.unwrap().upload_token(&login, &usermanager.users).await;

            Custom(Status::Ok, Ok(login.token.0))
        }
        Ok(_) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::NotFound,
                "User could not be found.".to_string(),
            ))
            .into()),
        ),
        Err(_err) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::InternalServerError,
                "A database error occured.".to_string(),
            ))
            .into()),
        ),
    }
}

pub async fn update_auth(
    id: String,
    login: Json<Login>,
    usermanager: &UserManager,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    match login.0 {
        Login::Credentials(credentials) => {
            match usermanager
                .update_auth(
                    id,
                    &Credentials {
                        email: credentials.email,
                        password: credentials.password,
                    }
                    .new_auth(),
                )
                .await
            {
                Ok(_) => Custom(Status::Ok, Ok(Json(true))),
                Err(_) => Custom(
                    Status::Ok,
                    Err(RequestError::from(Custom(
                        Status::InternalServerError,
                        "A database error occured.".into(),
                    ))
                    .into()),
                ),
            }
        }
        _ => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::BadRequest,
                "Credentials is the only login method currently supported".into(),
            ))
            .into()),
        ),
    }
}

pub async fn register(
    auth: Authentication,
    ip: String,
    usermanager: &UserManager,
) -> Custom<Result<String, Json<RequestError>>> {
    let result = auth
        .register(
            Server::current_time(),
            Server::generate_unique_id().to_string(),
            &usermanager.users,
        )
        .await;
    match result {
        Ok(user) if user.is_some() => {
            let user = user.unwrap();
            let login = database::login::Login::new(ip, Server::current_time(), auth);

            user.upload_token(&login, &usermanager.users).await;

            Custom(Status::Ok, Ok(login.token.0))
        }
        Ok(_) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(Status::Conflict, "User already exists.".into())).into()),
        ),
        Err(error) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(Status::InternalServerError, error)).into()),
        ),
    }
}
