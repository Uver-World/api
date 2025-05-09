use database::{user::User, Database};
use rocket::{get, http::Status, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

/// Retrieve the user informations from its token
#[openapi(tag = "Users")]
#[get("/token/<token>")] // <- route attribute
pub async fn from_token(
    user_data: UserData,
    database: &State<Database>,
    token: String,
) -> Custom<Result<Json<User>, Json<RequestError>>> {

    match database.user_manager.from_token(&token).await {
        Ok(user) if user.is_some() => Custom(Status::Ok, Ok(Json(user.unwrap()))),
        _ => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::NotFound,
                format!("User not found with token: {token}"),
            ))
            .into()),
        ),
    }
}

#[cfg(test)]
mod tests {

    use database::{user::User, Database};
    use rocket::http::{Method, Status};

    use crate::{
        testing::{self, dispatch_request, run_test},
        RequestError,
    };

    #[rocket::async_test]
    async fn test_from_token() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/user/token/{}", test_user.get_token().unwrap()),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let user = response.into_json::<User>().await.unwrap();
            assert_eq!(user.unique_id, test_user.unique_id);
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_from_unknown_token() {
        run_test(|client| async move {
            let request_user =
                testing::get_user(client.rocket().state::<Database>().unwrap())
                    .await;
            let request_token = request_user.get_token().unwrap();
            let token = "NO_TOKEN";

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/user/token/{token}"),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let request_error = response.into_json::<RequestError>().await.unwrap();
            assert_eq!(request_error.code, 404);
            assert_eq!(
                request_error.message,
                format!("User not found with token: {token}")
            );
        })
        .await;
    }

    #[rocket::async_test]
    async fn unauthorized_test_from_token() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/user/token/{}", test_user.get_token().unwrap()),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Unauthorized);
        })
        .await;
    }

    #[rocket::async_test]
    async fn forbidden_test_from_token() {
        run_test(|client| async move {
            let test_user =
                testing::get_user(client.rocket().state::<Database>().unwrap()).await;

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/user/token/{}", test_user.get_token().unwrap()),
                None,
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);
        })
        .await;
    }
}
