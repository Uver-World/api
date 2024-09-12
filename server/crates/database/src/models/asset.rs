use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::comment::Comment;

#[derive(Deserialize, Serialize, Debug, JsonSchema, Clone)]
pub struct Asset {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub upload_date: String,
    pub price: String,
    pub cover_image: String,
    pub images: Vec<String>,
    pub comments: Vec<Comment>,
    pub upvote_user_ids: Vec<u32>,
    pub downvote_user_ids: Vec<u32>,
    pub favorite_user_ids: Vec<u32>,
}

impl Asset {
    pub fn add_upvote(&mut self, user_id: u32) {
        if !self.upvote_user_ids.contains(&user_id) {
            self.upvote_user_ids.push(user_id);
        }
    }

    pub fn add_downvote(&mut self, user_id: u32) {
        if !self.downvote_user_ids.contains(&user_id) {
            self.downvote_user_ids.push(user_id);
        }
    }

    pub fn add_comment(&mut self, comment: Comment) {
        self.comments.push(comment);
    }
}
