use cookie::{Cookie, CookieBuilder};

use crate::TOKEN_DURATION;

pub enum CookieType {
    Authorization,
    Session,
}

pub fn create_cookie<'a>(token: &'a str) -> Cookie<'a> {
    CookieBuilder::new("Authorization", token)
        .max_age(TOKEN_DURATION)
        .same_site(cookie::SameSite::None)
        .http_only(true)
        .secure(true)
        .finish()
}

pub fn generate_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
