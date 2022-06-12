use crate::models::user::User;
use jsonwebtoken;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthResponse {
    ok: bool,
    user: User,
    access_token: String,
    exp_in: u64,
}

impl AuthResponse {
    pub fn new(ok: bool, user: User, access_token: String, exp_in: u64) -> Self {
        Self {
            ok,
            user,
            access_token,
            exp_in
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
}


impl Claims {
    pub fn new(sub: String, iat: u64, exp: u64) -> Self {
        let iat = iat;
        let exp = exp;
        Self {sub, iat, exp}
    }
}
