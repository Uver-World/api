use database::{group::Group, Database};
use rocket::{delete, http::Status, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

/// Delete the user linked to the token
#[openapi(tag = "Users")]
#[delete("/token/<token>")] // <- route attribute
pub async fn delete_from_token(
    user_data: UserData,
    database: &State<Database>,
    token: String,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::Website]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }
    match database.user_manager.delete_user(None, Some(&token)).await {
        Ok(Some(_)) => Custom(Status::Ok, Ok(Json(true))),
        Ok(_) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::InternalServerError,
                "A database error occured.".to_string(),
            ))
            .into()),
        ),
        Err(err) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(Status::NotFound, err)).into()),
        ),
    }
}

#[cfg(test)]
mod tests {

    use database::{group::Group, Database};
    use rocket::http::{Method, Status};

    use crate::{
        testing::{self, dispatch_request, run_test},
        RequestError,
    };

    #[rocket::async_test]
    async fn test_delete_from_token() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let website_user = testing::get_user(database, Group::Website).await;
            let website_token = website_user.get_token().unwrap();
            let user_token = test_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Delete,
                format!("/user/token/{}", user_token),
                None,
                Some(website_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<bool>().await.unwrap();
            assert_eq!(response, true);
            // User should have been deleted, so none should be found
            assert!(database
                .user_manager
                .from_token(user_token)
                .await
                .unwrap()
                .is_none());
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_unknown_delete_from_token() {
        run_test(|client| async move {
            let request_user =
                testing::get_user(client.rocket().state::<Database>().unwrap(), Group::Website)
                    .await;
            let request_token = request_user.get_token().unwrap();
            let token = "NO_TOKEN";

            let response = dispatch_request(
                &client,
                Method::Delete,
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
    async fn unauthorized_test_delete_from_token() {
        _unauthorized_test_delete_from_token(Group::User).await;
        _unauthorized_test_delete_from_token(Group::Server).await;
    }

    async fn _unauthorized_test_delete_from_token(request_group: Group) {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let request_user = testing::get_user(database, request_group).await;
            let request_token = request_user.get_token().unwrap();
            let user_token = test_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Delete,
                format!("/user/token/{}", user_token),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Unauthorized);

            // User should still exist in the database.
            assert!(database
                .user_manager
                .from_token(user_token)
                .await
                .unwrap()
                .is_some());
        })
        .await;
    }

    #[rocket::async_test]
    async fn forbidden_test_update() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let user_token = test_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Delete,
                format!("/user/token/{}", user_token),
                None,
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);
            // User should still exist in the database.
            assert!(database
                .user_manager
                .from_token(user_token)
                .await
                .unwrap()
                .is_some());
        })
        .await;
    }
}
