use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: String,
    pub post_slug: String,
    pub author_name: String,
    pub author_email: Option<String>,
    pub content: String,
    pub is_approved: bool,
    pub created_at: NaiveDateTime,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentForm {
    pub post_slug: String,
    pub author_name: String,
    pub author_email: Option<String>,
    pub content: String,
    pub honeypot: Option<String>, // Spam trap
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CommentQuery {
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub token: String,
}