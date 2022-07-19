use actix_web::{body::BoxBody, HttpResponse, HttpResponseBuilder as Response, ResponseError};
use reqwest::StatusCode;
use serde::Serialize;
use std::fmt::Display;
use thiserror::Error;

use crate::rps::models::RPSError;

/// The main wrapper for all the errors we can encounter
#[derive(Debug, Error)]
pub enum GlobalError {
    #[error("INTERNAL SERVER ERROR")]
    DieselError(diesel::result::Error),
    #[error("INTERNAL SERVER ERROR")]
    FileSystemError(std::io::Error),
    #[error("INTERNAL SERVER ERROR")]
    JwtError(jsonwebtoken::errors::Error),
    #[error("INTERNAL SERVER ERROR")]
    BcryptError(bcrypt::BcryptError),
    #[error("INTERNAL SERVER ERROR")]
    SerdeError(serde_json::error::Error),
    #[error("INTERNAL SERVER ERROR")]
    ActixError(actix_web::Error),
    #[error("INTERNAL SERVER ERROR")]
    R2D2Error,
    #[error("`{0}`")]
    AuthenticationError(AuthenticationError),
    #[error("`{0}`")]
    RPSError(RPSError)
}

impl GlobalError {
    /// Returns error description
    pub fn message(&self) -> String {
        match self {
            Self::AuthenticationError(e) => match e {
                AuthenticationError::UserNotFound => "Not found".to_string(),
                AuthenticationError::UserAlreadyExists => "Username taken".to_string(),
                AuthenticationError::BadPassword => "Invalid credentials".to_string(),
                AuthenticationError::InvalidToken => "Token either missing or expired".to_string(),
            },
            _ => "Internal server error".to_string(),
        }
    }

    /// Generates an http response with the given error
    pub fn respond(error: GlobalError) -> HttpResponse {
        let status = error.status_code();
        let error_response = ErrorResponse {
            code: status.as_u16(),
            error: error.to_string(),
            message: error.message(),
        };
        Response::new(status).json(error_response)
    }
}

impl ResponseError for GlobalError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            Self::AuthenticationError(e) => e.status_code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let status = self.status_code();
        let error_response = ErrorResponse {
            code: status.as_u16(),
            error: self.to_string(),
            message: self.message(),
        };
        Response::new(status).json(error_response)
    }
}
#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}
impl Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "There was an error: {}", self)
    }
}

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("User not found")]
    UserNotFound,
    #[error("Username taken")]
    UserAlreadyExists,
    #[error("Invalid credentials")]
    BadPassword,
    #[error("Invalid token")]
    InvalidToken,
}

impl AuthenticationError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            Self::UserNotFound => StatusCode::NOT_FOUND,
            Self::UserAlreadyExists => StatusCode::CONFLICT,
            Self::BadPassword => StatusCode::UNAUTHORIZED,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
        }
    }
}

impl From<AuthenticationError> for GlobalError {
    fn from(error: AuthenticationError) -> GlobalError {
        GlobalError::AuthenticationError(error)
    }
}
impl From<actix_web::Error> for GlobalError {
    fn from(error: actix_web::Error) -> GlobalError {
        GlobalError::ActixError(error)
    }
}
impl From<serde_json::error::Error> for GlobalError {
    fn from(error: serde_json::error::Error) -> GlobalError {
        GlobalError::SerdeError(error)
    }
}
impl From<diesel::result::Error> for GlobalError {
    fn from(error: diesel::result::Error) -> GlobalError {
        GlobalError::DieselError(error)
    }
}
impl From<std::io::Error> for GlobalError {
    fn from(error: std::io::Error) -> GlobalError {
        GlobalError::FileSystemError(error)
    }
}
impl From<jsonwebtoken::errors::Error> for GlobalError {
    fn from(error: jsonwebtoken::errors::Error) -> GlobalError {
        GlobalError::JwtError(error)
    }
}
impl From<bcrypt::BcryptError> for GlobalError {
    fn from(error: bcrypt::BcryptError) -> GlobalError {
        GlobalError::BcryptError(error)
    }
}
