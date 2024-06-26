use database::{peer::Peer, Database};
use rocket::{http::Status, post, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError, Server};

/// Authenticate the server
#[openapi(tag = "Users")]
#[post("/server_authenticate")] // <- route attribute
pub async fn server_authenticate(
    user_data: UserData,
    database: &State<Database>,
) -> Custom<Result<Json<Peer>, Json<RequestError>>> {

    let server_unique_id = user_data.id.unwrap();

    match database.peers_manager.peers_exist(&server_unique_id).await {
        Ok(exists) if exists => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::Conflict,
                "The server is already authenticated.".to_string(),
            ))
            .into()),
        ),
        _ => {
            let server_peer = Peer {
                room_id: Server::generate_unique_id().to_string(),
                creation_date: Server::current_time().to_string(),
                signaling_hostname: "x2025uverworld1150768037000.francecentral.cloudapp.azure.com"
                    .to_string(),
                signaling_port: 3536,
                server_unique_id: server_unique_id,
            };
            match database.peers_manager.create_peer(&server_peer).await {
                Ok(_) => Custom(Status::Ok, Ok(Json(server_peer))),
                Err(_) => Custom(
                    Status::Ok,
                    Err(RequestError::from(Custom(
                        Status::InternalServerError,
                        "A database server occured.".to_string(),
                    ))
                    .into()),
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use database::{peer::Peer, Database};
    use rocket::http::{Method, Status};

    use crate::{
        testing::{self, dispatch_request, run_test},
        RequestError,
    };

    #[rocket::async_test]
    async fn test_server_authenticate() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/server_authenticate"),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let generated_peer = response.into_json::<Peer>().await.unwrap();
            assert_eq!(generated_peer.server_unique_id, request_user.unique_id);
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_server_authenticate_conflict() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/server_authenticate"),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let generated_peer = response.into_json::<Peer>().await.unwrap();
            assert_eq!(generated_peer.server_unique_id, request_user.unique_id);

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/server_authenticate"),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<RequestError>().await.unwrap();
            assert_eq!(response.code, 409);
            assert_eq!(response.message, "The server is already authenticated.");
        })
        .await;
    }

    #[rocket::async_test]
    async fn unauthorized_test_server_authenticate() {
        _unauthorized_test_server_authenticate().await;
        _unauthorized_test_server_authenticate().await;
    }

    async fn _unauthorized_test_server_authenticate() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/server_authenticate"),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Unauthorized);
        })
        .await;
    }

    #[rocket::async_test]
    async fn forbidden_test_server_authenticate() {
        run_test(|client| async move {
            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/server_authenticate"),
                None,
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);
        })
        .await;
    }
}
