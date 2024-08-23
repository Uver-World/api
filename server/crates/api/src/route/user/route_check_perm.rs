use database::Database;
use rocket::{get, http::Status, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

// A route to verify if a user has a specific permission.
#[openapi(tag = "Users")]
#[get("/check-permission/<user_id>/permissions/<permission_name>")]
pub async fn check_perm(
    user_data: UserData,
    database: &State<Database>,
    user_id: String,
    permission_name: String,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    if user_data.id.is_none() {
        return Custom(
            Status::Unauthorized,
            Err(RequestError::from(Custom(
                Status::Unauthorized,
                format!("Token is invalid"),
            ))
            .into()),
        );
    }

    if !database
        .user_manager
        .user_exists(&user_id)
        .await
    {
        return Custom(
            Status::NotFound,
            Err(RequestError::from(Custom(
                Status::NotFound,
                format!("User not found"),
            ))
            .into()),
        );
    }

    let permission_id = match database
        .permission_manager
        .get_permission_id(&permission_name)
        .await
    {
        Ok(id) => id,
        Err(_) => {
            return Custom(
                Status::NotFound,
                Err(RequestError::from(Custom(
                    Status::NotFound,
                    format!("Unknown permission"),
                ))
                .into()),
            )
        }
    };

    let permission_check = match database
        .permission_manager
        .get_permission_id("permission.see")
        .await
    {
        Ok(id) => id,
        Err(_) => {
            return Custom(
                Status::NotFound,
                Err(RequestError::from(Custom(
                    Status::NotFound,
                    format!("Unknown permission"),
                ))
                .into()),
            )
        }
    };

    if !database.user_manager.has_permission(&user_data.id.unwrap(), &permission_check).await {
        return Custom(
            Status::Forbidden,
            Err(RequestError::from(Custom(
                Status::Forbidden,
                format!("Permission denied"),
            ))
            .into()),
        );
    }

    let has_perm = database
        .user_manager
        .has_permission(&user_id, &permission_id)
        .await;

    Custom(
        Status::Ok,
        Ok(Json(has_perm)),
    )
}

#[cfg(test)]
mod tests {
    use database::{authentication::Authentication, Database};
    use rocket::{http::{Method, Status}, local::asynchronous::LocalResponse};

    use crate::testing::{self, dispatch_request, run_test};

    #[rocket::async_test]
    async fn test_check_perm() {
        run_test(|rocket | async move {
            let database = rocket.rocket().state::<Database>().unwrap();
            let test_permission = database.permission_manager.get_permission_id("permission.see").await.unwrap();
            let test_user = testing::create_user(database, Authentication::None, vec![test_permission.clone()]).await;
            let request_token = test_user.get_token().unwrap();
            
            let response = dispatch_request(
                &rocket,
                Method::Get,
                format!("/user/check-permission/{}/permissions/{}", test_user.unique_id, test_permission.clone()),
                None,
                Some(request_token.to_string()),
            ).await;

            assert_eq!(response.status(), Status::Ok);
        }).await;
    }

    #[rocket::async_test]
    async fn test_check_perm_unauthorized() {
        run_test(|rocket | async move {
            let database = rocket.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let test_permission = testing::create_permission(database, "test_perm").await;
            
            let response = dispatch_request(
                &rocket,
                Method::Get,
                format!("/user/check-permission/{}/permissions/{}", test_user.unique_id, test_permission.unique_id),
                None,
                None
            ).await;

            assert_eq!(response.status(), Status::Unauthorized);
        }).await;
    }

    #[rocket::async_test]
    async fn test_check_perm_user_not_found() {
        run_test(|rocket | async move {
            let database = rocket.rocket().state::<Database>().unwrap();
            let test_permission = database.permission_manager.get_permission_id("permission.see").await.unwrap();
            let test_user = testing::create_user(database, Authentication::None, vec![test_permission.clone()]).await;
            let request_token = test_user.get_token().unwrap();

            let response = dispatch_request(
                &rocket,
                Method::Get,
                format!("/user/check-permission/{}/permissions/{}", "invalid", test_permission.clone()),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::NotFound);
        }).await;
    }

    #[rocket::async_test]
    async fn test_check_perm_permission_not_found() {
        run_test(|rocket | async move {
            let database = rocket.rocket().state::<Database>().unwrap();
            let test_permission = database.permission_manager.get_permission_id("permission.see").await.unwrap();
            let test_user = testing::create_user(database, Authentication::None, vec![test_permission.clone()]).await;
            let request_token = test_user.get_token().unwrap();

            let response = dispatch_request(
                &rocket,
                Method::Get,
                format!("/user/check-permission/{}/permissions/{}", test_user.unique_id, "invalid"),
                None,
                Some(request_token.to_string()),

            )
            .await;

            assert_eq!(response.status(), Status::NotFound);
        }).await;
    }
}
