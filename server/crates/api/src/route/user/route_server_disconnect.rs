use database::{group::Group, Database};
use rocket::{http::Status, post, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

/// Authenticate the server
#[openapi(tag = "Users")]
#[post("/server_disconnect")] // <- route attribute
pub async fn server_disconnect(
    user_data: UserData,
    database: &State<Database>,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::Server]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }
    let server_unique_id = user_data.id.unwrap();

    match database.peers_manager.peers_exist(&server_unique_id).await {
        Ok(exists) if !exists => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::NotModified,
                "The server is not connected.".to_string(),
            ))
            .into()),
        ),
        _ => match database.peers_manager.delete_peer(&server_unique_id).await {
            Ok(Some(result)) if result.deleted_count > 0 => Custom(Status::Ok, Ok(Json(true))),
            Ok(_) => Custom(
                Status::Ok,
                Err(RequestError::from(Custom(
                    Status::NotFound,
                    format!("Server not found with id: {server_unique_id}"),
                ))
                .into()),
            ),
            Err(_) => Custom(
                Status::Ok,
                Err(RequestError::from(Custom(
                    Status::InternalServerError,
                    "A database error occured.".into(),
                ))
                .into()),
            ),
        },
    }
}

#[cfg(test)]
mod tests {

    use database::{group::Group, peer::Peer, Database};
    use rocket::http::{Method, Status};

    use crate::{
        testing::{self, dispatch_request, run_test},
        Server,
    };

    #[rocket::async_test]
    async fn test_server_disconnect() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_server = testing::get_user(database, Group::Server).await;
            let test_user = testing::get_user(database, Group::User).await;
            let _test_org =
                testing::create_org(database, &test_user, vec![test_server.unique_id.clone()]);

            let server_peer = Peer {
                room_id: Server::generate_unique_id().to_string(),
                creation_date: Server::current_time().to_string(),
                signaling_hostname: "127.0.0.1".to_string(),
                signaling_port: 3536,
                server_unique_id: test_server.unique_id.clone(),
            };

            let _ = database
                .peers_manager
                .create_peer(&server_peer)
                .await
                .unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/server_disconnect"),
                None,
                Some(test_server.get_token().unwrap().to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<bool>().await.unwrap();
            assert!(response);
        })
        .await;
    }
}
