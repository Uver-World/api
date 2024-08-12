use database::{user::UserUpdate, Database};
use rocket::{http::Status, patch, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

/// Update the user informations from its token
#[openapi(tag = "Users")]
#[patch("/token/<token>", data = "<user_update>", format = "application/json")] // <- route attribute
pub async fn update(
    _user_data: UserData,
    database: &State<Database>,
    token: String,
    user_update: Json<Vec<UserUpdate>>,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {

    match database.user_manager.from_token(&token).await {
        Ok(user) if user.is_some() => {
            let uuid = user.unwrap().unique_id;
            match database.user_manager.update_user(uuid, user_update.0).await {
                Ok(_) => Custom(Status::Ok, Ok(Json(true))),
                Err(err) => Custom(
                    Status::Ok,
                    Err(
                        RequestError::from(Custom(Status::InternalServerError, err.to_string()))
                            .into(),
                    ),
                ),
            }
        }

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

    use database::{
        user::{User, UserUpdate},
        Database,
    };
    use rocket::http::{Method, Status};

    use crate::{
        testing::{self, dispatch_request, run_test},
        RequestError,
    };

    #[rocket::async_test]
    async fn test_update() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let user_token = test_user.get_token().unwrap();
            let updates = vec![UserUpdate::Username("Another username".to_string())];

            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/user/token/{}", user_token),
                Some(serde_json::to_string(&updates).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<bool>().await.unwrap();
            assert_eq!(response, true);
            let updated_user = database
                .user_manager
                .from_token(user_token)
                .await
                .unwrap()
                .unwrap();

            check_user_difference(&test_user, &updated_user)
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_unknown_update() {
        run_test(|client| async move {
            let request_user =
                testing::get_user(client.rocket().state::<Database>().unwrap())
                    .await;
            let request_token = request_user.get_token().unwrap();
            let token = "NO_TOKEN";
            let updates = vec![UserUpdate::Username("Another username".to_string())];

            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/user/token/{token}"),
                Some(serde_json::to_string(&updates).unwrap()),
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
    async fn unauthorized_test_update() {
        _unauthorized_test_update().await;
        _unauthorized_test_update().await;
    }

    async fn _unauthorized_test_update() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let user_token = test_user.get_token().unwrap();
            let updates = vec![UserUpdate::Username("Another username".to_string())];

            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/user/token/{}", user_token),
                Some(serde_json::to_string(&updates).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Unauthorized);
            check_user_difference(
                &test_user,
                &database
                    .user_manager
                    .from_token(&user_token)
                    .await
                    .unwrap()
                    .unwrap(),
            )
        })
        .await;
    }

    #[rocket::async_test]
    async fn forbidden_test_update() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let user_token = test_user.get_token().unwrap();
            let updates = vec![UserUpdate::Username("Another username".to_string())];

            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/user/token/{}", user_token),
                Some(serde_json::to_string(&updates).unwrap()),
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);
            check_user_difference(
                &test_user,
                &database
                    .user_manager
                    .from_token(&user_token)
                    .await
                    .unwrap()
                    .unwrap(),
            )
        })
        .await;
    }

    /// Check if the user has changed between the request and after it.
    /// If is_same is set false, then it will assert_ne! instead of asserting equal.
    fn check_user_difference(user1: &User, user2: &User) {
        // first we're checking if they're of the same id.
        assert_eq!(user1.unique_id, user2.unique_id);
    }
}
