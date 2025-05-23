use database::{Database};
use rocket::{delete, http::Status, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

/// Delete the user from its id.
#[openapi(tag = "Users")]
#[delete("/id/<id>")] // <- route attribute
pub async fn delete_from_id(
    _user_data: UserData,
    database: &State<Database>,
    id: String,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {

    match database.user_manager.delete_user(Some(&id), None).await {
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

    use database::{Database};
    use rocket::http::{Method, Status};

    use crate::{
        testing::{self, dispatch_request, run_test},
        RequestError,
    };

    #[rocket::async_test]
    async fn test_delete_from_id() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Delete,
                format!("/user/id/{}", test_user.unique_id),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<bool>().await.unwrap();
            assert_eq!(response, true);
            // User should have been deleted, so none should be found
            assert!(database
                .user_manager
                .from_id(&test_user.unique_id)
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
                testing::get_user(client.rocket().state::<Database>().unwrap())
                    .await;
            let request_token = request_user.get_token().unwrap();
            let id = "NO_ID";

            let response = dispatch_request(
                &client,
                Method::Delete,
                format!("/user/id/{}", id),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let request_error = response.into_json::<RequestError>().await.unwrap();
            assert_eq!(request_error.code, 404);
            assert_eq!(
                request_error.message,
                format!("User not found with id: {}", id)
            );
        })
        .await;
    }

    #[rocket::async_test]
    async fn unauthorized_test_delete_from_id() {
        _unauthorized_test_delete_from_id().await;
        _unauthorized_test_delete_from_id().await;
    }

    async fn _unauthorized_test_delete_from_id() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database, ).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Delete,
                format!("/user/id/{}", test_user.unique_id),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Unauthorized);

            // User should still exist in the database.
            assert!(database
                .user_manager
                .from_id(&test_user.unique_id)
                .await
                .unwrap()
                .is_some());
        })
        .await;
    }

    #[rocket::async_test]
    async fn forbidden_test_delete_from_id() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;

            let response = dispatch_request(
                &client,
                Method::Delete,
                format!("/user/id/{}", test_user.unique_id),
                None,
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);
            // User should still exist in the database.
            assert!(database
                .user_manager
                .from_id(&test_user.unique_id)
                .await
                .unwrap()
                .is_some());
        })
        .await;
    }
}
