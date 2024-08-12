use database::Database;
use rocket::{delete, http::Status, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

// A route to remove a permission to a user.
#[openapi(tag = "Users")]
#[delete("/<user_id>/permissions/<permission_id>")]
pub async fn remove_perm(
    user_data: UserData,
    database: &State<Database>,
    user_id: String,
    permission_id: String,
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

    if !database
        .permission_manager
        .permission_exists(&permission_id)
        .await
    {
        return Custom(
            Status::NotFound,
            Err(RequestError::from(Custom(
                Status::NotFound,
                format!("Permission not found"),
            ))
            .into()),
        );
    }

    let permission_remove = match database
        .permission_manager
        .get_permission_id("permission.remove")
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

    if !database.user_manager.has_permission(&user_data.id.unwrap(), &permission_remove).await {
        return Custom(
            Status::Forbidden,
            Err(RequestError::from(Custom(
                Status::Forbidden,
                format!("Permission denied"),
            ))
            .into()),
        );
    }

    let permission_id_clone = permission_id.clone();
    
    match database
        .user_manager
        .remove_permission(user_id, permission_id_clone)
        .await
    {
        Ok(_) => Custom(Status::Created, Ok(Json(true))),
        Err(_err) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::NotFound,
                format!("User not found"),
            ))
            .into()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use database::{authentication::Authentication, server::Server, Database};
    use rocket::{data, http::{Method, Status}};

    use crate::testing::{self, dispatch_request, run_test};

    #[rocket::async_test]
    async fn test_remove_perm() {
        run_test(|rocket | async move {
            let database = rocket.rocket().state::<Database>().unwrap();
            let test_permission = database.permission_manager.get_permission_id("permission.remove").await.unwrap();
            let test_user = testing::create_user(database, Authentication::None, vec![test_permission.clone()]).await;
            let request_token = test_user.get_token().unwrap();
            
            let response = dispatch_request(
                &rocket,
                Method::Delete,
                format!("/user/{}/permissions/{}", test_user.unique_id, test_permission.clone()),
                None,
                Some(request_token.to_string()),
            ).await;

            assert_eq!(response.status(), Status::Created);
        }).await;
    }

    #[rocket::async_test]
    async fn test_remove_perm_unauthorized() {
        run_test(|rocket | async move {
            let database = rocket.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let test_permission = testing::create_permission(database, "test_perm").await;
            
            let response = dispatch_request(
                &rocket,
                Method::Delete,
                format!("/user/{}/permissions/{}", test_user.unique_id, test_permission.unique_id),
                None,
                None
            ).await;

            assert_eq!(response.status(), Status::Unauthorized);
        }).await;
    }

    #[rocket::async_test]
    async fn test_remove_perm_user_not_found() {
        run_test(|rocket | async move {
            let database = rocket.rocket().state::<Database>().unwrap();
            let test_permission = database.permission_manager.get_permission_id("permission.remove").await.unwrap();
            let test_user = testing::create_user(database, Authentication::None, vec![test_permission.clone()]).await;
            let request_token = test_user.get_token().unwrap();

            let response = dispatch_request(
                &rocket,
                Method::Delete,
                format!("/user/{}/permissions/{}", "invalid", test_permission.clone()),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::NotFound);
        }).await;
    }

    #[rocket::async_test]
    async fn test_remove_perm_permission_not_found() {
        run_test(|rocket | async move {
            let database = rocket.rocket().state::<Database>().unwrap();
            let test_permission = database.permission_manager.get_permission_id("permission.remove").await.unwrap();
            let test_user = testing::create_user(database, Authentication::None, vec![test_permission.clone()]).await;
            let request_token = test_user.get_token().unwrap();

            let response = dispatch_request(
                &rocket,
                Method::Delete,
                format!("/user/{}/permissions/{}", test_user.unique_id, "invalid"),
                None,
                Some(request_token.to_string()),

            )
            .await;

            assert_eq!(response.status(), Status::NotFound);
        }).await;
    }
}
