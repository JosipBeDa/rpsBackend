use crate::models::authentication::AuthenticationError;
use actix_web::ResponseError;
use std::fmt::Display;

#[derive(Debug)]
pub enum CustomError {
    DieselError(diesel::result::Error),
    FileSystemError(std::io::Error),
    JwtError(jsonwebtoken::errors::Error),
    BcryptError(bcrypt::BcryptError),
    R2D2Error,
    AlreadyExistsError,
    AuthenticationError(AuthenticationError),
    SerdeError(serde_json::error::Error),
    ActixError(actix_web::Error),
}

impl From<AuthenticationError> for CustomError {
    fn from(error: AuthenticationError) -> CustomError {
        CustomError::AuthenticationError(error)
    }
}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "There was an error: {}", self)
    }
}

impl From<actix_web::Error> for CustomError {
    fn from(error: actix_web::Error) -> CustomError {
        CustomError::ActixError(error)
    }
}

impl From<serde_json::error::Error> for CustomError {
    fn from(error: serde_json::error::Error) -> CustomError {
        CustomError::SerdeError(error)
    }
}

impl From<diesel::result::Error> for CustomError {
    fn from(error: diesel::result::Error) -> CustomError {
        CustomError::DieselError(error)
    }
}
impl From<std::io::Error> for CustomError {
    fn from(error: std::io::Error) -> CustomError {
        CustomError::FileSystemError(error)
    }
}
impl From<jsonwebtoken::errors::Error> for CustomError {
    fn from(error: jsonwebtoken::errors::Error) -> CustomError {
        CustomError::JwtError(error)
    }
}
impl From<bcrypt::BcryptError> for CustomError {
    fn from(error: bcrypt::BcryptError) -> CustomError {
        CustomError::BcryptError(error)
    }
}

impl ResponseError for CustomError {}
