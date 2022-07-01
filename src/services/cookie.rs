use cookie::{Cookie, CookieBuilder};

pub enum CookieType {
    Authorization,
    Session,
}

pub fn create_cookie<'a>(token: &'a str, ct: CookieType) -> Cookie<'a> {
    match ct {
        CookieType::Authorization => CookieBuilder::new("Authorization", token)
            .max_age(time::Duration::hours(2))
            .same_site(cookie::SameSite::None)
            .http_only(true)
            .secure(true)
            .finish(),
        CookieType::Session => CookieBuilder::new("session_id", token)
            .max_age(time::Duration::hours(2))
            .same_site(cookie::SameSite::None)
            .http_only(false)
            .secure(false)
            .finish(),
    }
}

pub fn generate_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
