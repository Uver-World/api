use database::{group::Group, Database};
use database::license::License;
use rocket::get;
use rocket::{http::Status, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;
use crate::{model::user_token::UserData, RequestError};


/// Get all licenses of a user
///
/// Requires 'Website', 'Server', 'User', 'Guest' group
#[openapi(tag = "Users")]
#[get("/id/<id>/license")]
pub async fn get_licenses(
    user_data: UserData,
    database: &State<Database>,
    id: String,
) -> Custom<Result<Json<Vec<License>>, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::Website, Group::Server, Group::User]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }

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

    let user = user.unwrap();


    let licenses = match database.license_manager.get_licenses(&user.unique_id).await {
        Ok(licenses) => licenses,
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

    Custom(Status::Ok, Ok(Json(licenses)))
}

#[cfg(test)]
mod tests {

    use database::{group::Group, Database, license::License};
    use rocket::http::{Method, Status};

    use crate::testing::{self, dispatch_request, run_test};

    #[rocket::async_test]
    async fn test_create_license() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let request_user = testing::get_user(database, Group::Website).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/{}/license", test_user.unique_id),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Created);
            let license = response.into_json::<License>().await.unwrap();
            assert_eq!(license.user_id, test_user.unique_id);
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_create_license_unknown_user() {
        run_test(|client| async move {
            let request_user =
                testing::get_user(client.rocket().state::<Database>().unwrap(), Group::Website)
                    .await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/unknow/license"),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::NotFound);
        })
        .await;
    }
}