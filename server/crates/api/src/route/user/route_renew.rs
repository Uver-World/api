use database::{authentication::Authentication, group::Group, Database};
use rocket::{post, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{
    model::{api_socket_addr::ApiSocketAddr, login::Login, user_token::UserData},
    RequestError,
};

use super::helper;

/// Renew an user token with either the user credentials, or with the serverid
///
/// To regenerate a server's token, you have to be part of the Website group
#[openapi(tag = "Users")]
#[post("/renew", data = "<login>", format = "application/json")] // <- route attribute
pub async fn renew(
    user_data: UserData,
    database: &State<Database>,
    login: Json<Login>,
    remot_addr: ApiSocketAddr,
) -> Custom<Result<String, Json<RequestError>>> {
    let ip = remot_addr.0.ip().to_string();
    match login.0 {
        Login::Credentials(credentials) => {
            let auth = Authentication::Credentials(credentials);
            let user = auth.get(&database.user_manager.users).await;
            helper::renew_token(user, ip, auth, &database.user_manager).await
        }
        Login::UserId(user_id) => {
            if let Err(response) = user_data.matches_group(vec![Group::Website]) {
                return Custom(response.0, Err(RequestError::from(response).into()));
            }
            let user = database.user_manager.from_id(&user_id).await;
            helper::renew_token(
                user.map_err(|err| err.to_string()),
                ip,
                Authentication::None,
                &database.user_manager,
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests {

    use database::{
        authentication::{Authentication, Credentials},
        group::Group,
        Database,
    };
    use rocket::http::{Method, Status};

    use crate::{
        model::login::Login,
        testing::{self, dispatch_request, run_test},
        RequestError,
    };

    #[rocket::async_test]
    async fn test_renew_admin() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let server_user = testing::get_user(database, Group::Server).await;
            let request_user = testing::get_user(database, Group::Website).await;
            let request_token = request_user.get_token().unwrap();

            println!(
                "body = {:?}",
                serde_json::to_string(&Login::UserId(server_user.unique_id.clone()))
            );

            let response = dispatch_request(
                &client,
                Method::Post,
                "/user/renew".to_string(),
                Some(serde_json::to_string(&Login::UserId(server_user.unique_id.clone())).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_string().await.unwrap();
            // We should find the same id from the token that we received
            assert_eq!(
                database
                    .user_manager
                    .from_token(&response)
                    .await
                    .unwrap()
                    .unwrap()
                    .unique_id,
                server_user.unique_id
            );
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_renew() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let credentials = Credentials {
                email: "test@test.fr".to_string(),
                password: "test".to_string(),
            };
            let test_user = testing::create_user(
                database,
                Group::User,
                Authentication::Credentials(credentials.clone()),
            )
            .await;
            let response = dispatch_request(
                &client,
                Method::Post,
                "/user/renew".to_string(),
                Some(serde_json::to_string(&Login::Credentials(credentials)).unwrap()),
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_string().await.unwrap();
            // We should find the same id from the token that we received
            assert_eq!(
                database
                    .user_manager
                    .from_token(&response)
                    .await
                    .unwrap()
                    .unwrap()
                    .unique_id,
                test_user.unique_id
            );
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_unknown_admin_renew() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database, Group::Website).await;
            let request_token = request_user.get_token().unwrap();
            let id = "NO_ID".to_string();

            let response = dispatch_request(
                &client,
                Method::Post,
                "/user/renew".to_string(),
                Some(serde_json::to_string(&Login::UserId(id)).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let request_error = response.into_json::<RequestError>().await.unwrap();
            assert_eq!(request_error.code, 404);
            assert_eq!(request_error.message, format!("User could not be found."));
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_unknown_renew() {
        run_test(|client| async move {
            let credentials = Credentials {
                email: "test@test.fr".to_string(),
                password: "test".to_string(),
            };
            let response = dispatch_request(
                &client,
                Method::Post,
                "/user/renew".to_string(),
                Some(serde_json::to_string(&Login::Credentials(credentials)).unwrap()),
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let request_error = response.into_json::<RequestError>().await.unwrap();
            assert_eq!(request_error.code, 404);
            assert_eq!(request_error.message, format!("User could not be found."));
        })
        .await;
    }

    #[rocket::async_test]
    async fn unauthorized_test_renew() {
        _unauthorized_test_renew(Group::User).await;
        _unauthorized_test_renew(Group::Server).await;
    }

    async fn _unauthorized_test_renew(request_group: Group) {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database, request_group).await;
            let server_user = testing::get_user(database, Group::Server).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                "/user/renew".to_string(),
                Some(serde_json::to_string(&Login::UserId(server_user.unique_id.clone())).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Unauthorized);

            // The last token should not have been updated.
            assert_eq!(
                database
                    .user_manager
                    .from_id(&server_user.unique_id)
                    .await
                    .unwrap()
                    .unwrap()
                    .get_token()
                    .unwrap(),
                server_user.get_token().unwrap()
            );
        })
        .await;
    }

    #[rocket::async_test]
    async fn forbidden_test_renew_admin() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let server_user = testing::get_user(database, Group::Server).await;

            let response = dispatch_request(
                &client,
                Method::Post,
                "/user/renew".to_string(),
                Some(serde_json::to_string(&Login::UserId(server_user.unique_id.clone())).unwrap()),
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);

            // The last token should not have been updated.
            assert_eq!(
                database
                    .user_manager
                    .from_id(&server_user.unique_id)
                    .await
                    .unwrap()
                    .unwrap()
                    .get_token()
                    .unwrap(),
                server_user.get_token().unwrap()
            );
        })
        .await;
    }
}
