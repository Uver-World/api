use database::{Database};
use database::license::License;
use rocket::{http::Status, post, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;
use crate::{model::user_token::UserData, RequestError, Server};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;


/// Create a new license
///
/// Requires 'Website', 'Server', 'User', 'Guest' group
#[openapi(tag = "Users")]
#[post("/<id>/license")]
pub async fn create_license(
    _user_data: UserData,
    database: &State<Database>,
    id: String,
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

    let user = user.unwrap();

    let license = License {
        unique_id: Server::generate_unique_id().to_string(),
        user_id: user.unique_id.clone(),
        license: generate_license(),
    };

    // if user.group == Group::Guest {
    //     match database.user_manager.update_group(user.unique_id.clone()).await {
    //         Ok(_) => (),
    //         Err(_) => return Custom(
    //             Status::InternalServerError,
    //             Err(RequestError::from(Custom(
    //                 Status::InternalServerError,
    //                 "Failed to update user group.".to_string(),
    //             ))
    //             .into()),
    //         ),
    //     }
    // }
    

    match database.license_manager.create(&license).await {
        Ok(_) => Custom(Status::Created, Ok(Json(license))),
        Err(_) => Custom(
            Status::InternalServerError,
            Err(RequestError::from(Custom(
                Status::InternalServerError,
                "Failed to create license.".to_string(),
            ))
            .into()),
        ),
    }
}

fn generate_license() -> String {
    let rng = thread_rng();
    let alphanumeric: String = rng.sample_iter(&Alphanumeric).take(19).map(|x| x as char).collect();

    let formatted_license = format!("UVW-{}-{}-{}-{}", &alphanumeric[0..4], &alphanumeric[4..8], &alphanumeric[8..12], &alphanumeric[12..15]);
    formatted_license
}

#[cfg(test)]
mod tests {

    use database::{Database, license::License};
    use rocket::http::{Method, Status};

    use crate::testing::{self, dispatch_request, run_test};

    #[rocket::async_test]
    async fn test_create_license() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
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
                testing::get_user(client.rocket().state::<Database>().unwrap())
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