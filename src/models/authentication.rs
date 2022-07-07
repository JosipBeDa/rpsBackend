use serde::{Deserialize, Serialize};

use super::user::ChatUser;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthResponse {
    session_id: Option<String>,
    ok: bool,
    message: &'static str,
    user: Option<ChatUser>,
}
#[derive(Debug)]
pub enum AuthenticationError {
    UserNotFound,
    UserAlreadyExists,
    BadPassword,
    TokenExpired,
    Unauthorized
}
impl actix_web::ResponseError for AuthenticationError {}

impl std::fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AuthenticationError::UserNotFound => write!(f, "User doesn't exist"),
            AuthenticationError::UserAlreadyExists => write!(f, "Username already taken"),
            AuthenticationError::BadPassword => write!(f, "Bad credentials"),
            AuthenticationError::TokenExpired => write!(f, "Token expired"),
            AuthenticationError::Unauthorized => write!(f, "Bad token"),
        }
    }
}

impl AuthResponse {
    pub fn succeed(user: ChatUser, message: &'static str) -> Self {
        Self {
            session_id: Some(user.id.clone()),
            ok: true,
            message,
            user: Some(user),
        }
    }
    pub fn fail(message: &'static str) -> Self {
        Self {
            session_id: None,
            ok: false,
            message,
            user: None,
        }
    }
}
