use crate::models::authentication::{AuthForm, AuthResponse};
use crate::models::user::User;
use crate::services::cookie::*;
use crate::services::jwt;
use crate::state::app::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponseBuilder as Response, Responder};
use reqwest::StatusCode;


pub async fn handler(
    auth_form: web::Form<AuthForm>,
    state: web::Data<AppState>,
    session: Session,
) -> impl Responder {
    let db_connection = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return Response::new(StatusCode::INTERNAL_SERVER_ERROR).finish(),
    };
    // Find user in db
    match User::find_by_uname(&db_connection, &auth_form.username) {
        // Compare passwords
        Ok(Some(user)) => match bcrypt::verify(auth_form.password.clone(), &user.password) {
            Ok(verified) => {
                if !verified {
                    // Bad password
                    return Response::new(StatusCode::UNAUTHORIZED)
                        .json(AuthResponse::fail("Invalid Credentials"));
                }
                // Generate token and session ID
                if let Ok((token, _exp_in)) = jwt::generate_jwt(&user.id) {
                    session.insert("user_id", &user.id).unwrap();
                    session.insert("username", &user.username).unwrap();
                    let cookie = create_cookie(&token, CookieType::Authorization);
                    Response::new(StatusCode::OK)
                        .cookie(cookie)
                        .json(AuthResponse::succeed(user, "Success!"))
                } else {
                    // Failed to generate token
                    Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                        .json(AuthResponse::fail("Internal Server Error"))
                }
            }
            // Bcrypt error
            Err(_) => Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                .json(AuthResponse::fail("Internal Server Error")),
        },
        // User not found
        Ok(None) => Response::new(StatusCode::INTERNAL_SERVER_ERROR)
            .json(AuthResponse::fail("User not found")),
        // Diesel error
        Err(_) => Response::new(StatusCode::INTERNAL_SERVER_ERROR)
            .json(AuthResponse::fail("Internal Server Error")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_login() {
        let _ = User {
            id: "1".to_string(),
            username: "shiva".to_string(),
            password: "shiva".to_string(),
        };
    }
}
