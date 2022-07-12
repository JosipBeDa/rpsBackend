use actix_web::{HttpResponse, HttpResponseBuilder as Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use super::{
    error::GlobalError,
    user::{ChatUser, User},
};
use crate::services::{cookie::create_cookie, jwt};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthForm {
    pub username: String,
    pub password: String,
}

/// The main response sent to the client upon auth requests
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    session_id: Option<String>,
    ok: bool,
    message: &'static str,
    user: Option<ChatUser>,
    error_message: Option<String>,
}

impl AuthResponse {
    pub fn succeed_with_token(user: User) -> Result<HttpResponse, GlobalError> {
        let user = user.convert();
        match jwt::generate_jwt(user.to_string()) {
            Ok(token) => {
                let cookie = create_cookie(&token);
                Ok(Response::new(StatusCode::OK).cookie(cookie).json(Self {
                    session_id: Some(user.id.clone()),
                    ok: true,
                    message: "Success!",
                    user: Some(user),
                    error_message: None,
                }))
            }
            Err(e) => Err(e),
        }
    }
}
