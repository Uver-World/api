use database::{Database};
use rocket::{get, http::Status, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

/// Check if an email is registered or not
#[openapi(tag = "Users")]
#[get("/email_exists/<email>")] // <- route attribute
pub async fn email_exists(
    _user_data: UserData,
    database: &State<Database>,
    email: String,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {

    match database.user_manager.email_exists(email).await {
        Ok(value) => Custom(Status::Ok, Ok(Json(value))),
        _ => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::InternalServerError,
                "A database error occured.".to_string(),
            ))
            .into()),
        ),
    }
}

#[cfg(test)]
mod tests {

    use database::{
        authentication::{Authentication, Credentials},
        Database,
    };
    use rocket::http::{Method, Status};

    use crate::testing::{self, dispatch_request, run_test};

    #[rocket::async_test]
    async fn test_email_exists() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let credentials = Credentials {
                email: "test@test.fr".to_string(),
                username: Option::Some("test".to_string()),
                avatar: Option::Some("test".to_string()),
                password: "test".to_string(),
            };
            let _ = testing::create_user(
                database,
                Authentication::Credentials(credentials.clone()),
                Vec::new()
            )
            .await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/user/email_exists/{}", credentials.email),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<bool>().await.unwrap();
            assert!(response);
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_from_unknown_email_exists() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let email = "NO_EMAIL@EMAIL.FR".to_string();

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/user/email_exists/{}", email),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<bool>().await.unwrap();
            assert!(!response);
        })
        .await;
    }

    #[rocket::async_test]
    async fn unauthorized_test_email_exists() {
        _unauthorized_test_email_exists().await;
        _unauthorized_test_email_exists().await;
    }

    async fn _unauthorized_test_email_exists() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let credentials = Credentials {
                email: "test@test.fr".to_string(),
                username: Option::Some("test".to_string()),
                avatar: Option::Some("test".to_string()),
                password: "test".to_string(),
            };
            let _ = testing::create_user(
                database,
                Authentication::Credentials(credentials.clone()),
                Vec::new()
            )
            .await;

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/user/email_exists/{}", credentials.email),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Unauthorized);
        })
        .await;
    }

    #[rocket::async_test]
    async fn forbidden_test_email_exists() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let credentials = Credentials {
                email: "test@test.fr".to_string(),
                username: Option::Some("test".to_string()),
                avatar: Option::Some("test".to_string()),
                password: "test".to_string(),
            };
            let _ = testing::create_user(
                database,
                Authentication::Credentials(credentials.clone()),
                Vec::new()
            )
            .await;

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/user/email_exists/{}", credentials.email),
                None,
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);
        })
        .await;
    }
}
