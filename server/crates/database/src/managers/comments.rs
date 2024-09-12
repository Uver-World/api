use mongodb::{
    bson::{doc},
    error::Error,
    results::{DeleteResult, InsertOneResult},
    Collection,
};
use crate::models::comment::Comment;

pub struct CommentManager {
    pub comments: Collection<Comment>,
}

impl CommentManager {
    pub fn init(comments: Collection<Comment>) -> Self {
        Self { comments }
    }

    pub async fn add_comment(&self, comment: &Comment) -> Result<InsertOneResult, Error> {
        let result = self.comments.insert_one(comment, None).await?;
        Ok(result)
    }

    pub async fn get_comment_by_user(&self, user_id: u32) -> Result<Option<Comment>, Error> {
        match self.comments.find_one(doc! { "user_id": user_id }, None).await? {
            Some(comment) => Ok(Some(comment)),
            None => Ok(None),
        }
    }

    pub async fn delete_comment(&self, user_id: u32) -> Result<DeleteResult, Error> {
        let result = self.comments.delete_one(doc! { "user_id": user_id }, None).await?;
        Ok(result)
    }
    
    pub async fn comment_exists(&self, user_id: u32) -> Result<bool, Error> {
        let count = self.comments.count_documents(doc! { "user_id": user_id }, None).await?;
        Ok(count > 0)
    }
}

impl Clone for CommentManager {
    fn clone(&self) -> Self {
        Self {
            comments: self.comments.clone(),
        }
    }
}
