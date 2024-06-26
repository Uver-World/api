use database::license::License;
use database::{Database};
use rocket::post;
use rocket::{http::Status, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;
use crate::{model::user_token::UserData, RequestError};


/// Verify a license and if it is valid
///
/// Requires 'Website', 'Server', 'User', 'Guest' group
#[openapi(tag = "Users")]
#[post("/<id>/license/<license_id>")]
pub async fn check_licenses(
    user_data: UserData,
    database: &State<Database>,
    id: String,
    license_id: String,
) -> Custom<Result<Json<License>, Json<RequestError>>> {


    let user = match database.user_manager.from_id(&id).await {
        Ok(user) => user,
        Err(_) => return Custom(
            Status::InternalServerError,
            Err(RequestError::from(Custom(
                Status::InternalServerError,
                "Failed to retrieve user.".to_string(),
            ))
            .into()),
        ),
    };

    if user.is_none() {
        return Custom(
            Status::NotFound,
            Err(RequestError::from(Custom(
                Status::NotFound,
                format!("User not found with id: {id}"),
            ))
            .into()),
        );
    }

    let license = match database.license_manager.get_license(&license_id).await {
        Ok(license) => license,
        Err(err) => {
            return Custom(
                Status::InternalServerError,
                Err(RequestError::from(Custom(
                    Status::InternalServerError,
                    err.to_string(),
                ))
                .into()),
            )
        }
    };

    match license {
        Some(license) => {
            Custom(Status::Ok, Ok(Json(license)))
        }
        None => Custom(Status::Forbidden, Err(RequestError::from(Custom(
            Status::Forbidden,
            "License not valid.".to_string(),
        ))
        .into())),
    }
}

#[cfg(test)]
mod tests {

    use database::{Database, license::License};
    use rocket::http::{Method, Status};

    use crate::testing::{self, dispatch_request, run_test};

    #[rocket::async_test]
    async fn test_check_licenses() {
        run_test(|client | async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let license = License {
                unique_id: "test".to_string(),
                user_id: test_user.unique_id.clone(),
                license: "UVW-1234-5678-9012-345".to_string(),
            };

            database.license_manager.create(&license).await.unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/{}/license/{}", test_user.unique_id, license.license),
                None,
                Some(request_token.to_string()),
            ).await;

            println!("response {:?}", response);

            assert_eq!(response.status(), Status::Ok);
        }).await;
    }

    #[rocket::async_test]
    // Check an invalid license
    async fn test_check_licenses_invalid() {
        run_test(|client | async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let license = License {
                unique_id: "test".to_string(),
                user_id: test_user.unique_id.clone(),
                license: "UVW-1234-5678-9012-345".to_string(),
            };
            database.license_manager.create(&license).await.unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/{}/license/{}", test_user.unique_id, "invalid"),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);
        }).await;
    }

    #[rocket::async_test]
    async fn test_check_licenses_unknown_user() {
        run_test(|client | async move {
            let request_user =
                testing::get_user(client.rocket().state::<Database>().unwrap())
                    .await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/unknow/license/unknow"),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::NotFound);
        }).await;
    }
}