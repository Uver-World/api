// #[openapi(tag = "Comments")]
// #[delete("/<asset_id>/comment/<comment_id>")]
// pub async fn delete_comment(
//     database: &State<Database>,
//     asset_id: String,
//     comment_id: String,
// ) -> Custom<Result<String, Json<RequestError>>> {
//     let comment_manager = &database.comment_manager;

//     match comment_manager.delete_comment(&asset_id, &comment_id).await {
//         Ok(_) => Custom(Status::Ok, Ok(format!("Comment {} deleted from asset {}", comment_id, asset_id))),
//         Err(err) => Custom(
//             Status::InternalServerError,
//             Err(Json(RequestError::from(err.to_string()))),
//         ),
//     }
// }
