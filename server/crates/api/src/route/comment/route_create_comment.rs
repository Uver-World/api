// use crate::models::comment::Comment;

// #[openapi(tag = "Comments")]
// #[post("/<asset_id>/comment", data = "<new_comment>", format = "application/json")]
// pub async fn create_comment(
//     database: &State<Database>,
//     asset_id: String,
//     new_comment: Json<Comment>,
// ) -> Custom<Result<String, Json<RequestError>>> {
//     let comment_manager = &database.comment_manager;
//     let comment = new_comment.into_inner();

//     match comment_manager.add_comment(&asset_id, &comment).await {
//         Ok(_) => Custom(Status::Ok, Ok(format!("Comment added to asset {}", asset_id))),
//         Err(err) => Custom(
//             Status::InternalServerError,
//             Err(Json(RequestError::from(err.to_string()))),
//         ),
//     }
// }
