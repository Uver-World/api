use database::{authentication::Credentials, group::Group, managers::UserManager, Database};
use rocket::{http::Status, patch, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{
    model::{login::Login, user_token::UserData},
    RequestError,
};

/// Update the way an user authenticate itself
///
/// This update requires the user unique identifier
#[openapi(tag = "Users")]
#[patch("/update_auth", data = "<login>", format = "application/json")] // <- route attribute
pub async fn update_auth(
    user_data: UserData,
    database: &State<Database>,
    login: Json<Login>,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::User]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }
    _update_auth(user_data.id.unwrap(), login, &database.user_manager).await
}

async fn _update_auth(
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
                        username: credentials.username,
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

#[cfg(test)]
mod tests {

    use database::{
        authentication::{Authentication, Credentials},
        group::Group,
        Database,
    };
    use rocket::http::{Method, Status};

    use crate::{
        model::login::Login,
        testing::{self, dispatch_request, run_test},
        RequestError,
    };

    #[rocket::async_test]
    async fn test_update_auth() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let user_token = test_user.get_token().unwrap();
            let credentials = Credentials {
                email: "test@test.fr".to_string(),
                username: "test".to_string(),
                password: "test".to_string(),
            };

            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/user/update_auth"),
                Some(serde_json::to_string(&Login::Credentials(credentials.clone())).unwrap()),
                Some(user_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<bool>().await.unwrap();
            assert_eq!(response, true);
            let user = database
                .user_manager
                .from_id(&test_user.unique_id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                user.authentication,
                Authentication::Credentials(credentials)
            );
        })
        .await;
    }

    #[rocket::async_test]
    async fn unauthorized_test_update_auth() {
        _unauthorized_test_update_auth(Group::Server).await;
        _unauthorized_test_update_auth(Group::Website).await;
    }

    async fn _unauthorized_test_update_auth(request_group: Group) {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database, request_group).await;
            let request_token = request_user.get_token().unwrap();
            let credentials = Credentials {
                email: "test@test.fr".to_string(),
                username: "test".to_string(),
                password: "test".to_string(),
            };

            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/user/update_auth"),
                Some(serde_json::to_string(&Login::Credentials(credentials.clone())).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Unauthorized);
            let user = database
                .user_manager
                .from_id(&request_user.unique_id)
                .await
                .unwrap()
                .unwrap();
            assert_ne!(
                user.authentication,
                Authentication::Credentials(credentials)
            );
        })
        .await;
    }

    #[rocket::async_test]
    async fn incorrect_test_update_auth() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database, Group::User).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/user/update_auth"),
                Some(serde_json::to_string(&Login::UserId("NO_ID".to_string())).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<RequestError>().await.unwrap();
            assert_eq!(response.code, 400);
        })
        .await;
    }

    #[rocket::async_test]
    async fn forbidden_test_update_auth() {
        run_test(|client| async move {
            let credentials = Credentials {
                email: "test@test.fr".to_string(),
                username: "test".to_string(),
                password: "test".to_string(),
            };

            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/user/update_auth"),
                Some(serde_json::to_string(&Login::Credentials(credentials.clone())).unwrap()),
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);
        })
        .await;
    }
}
