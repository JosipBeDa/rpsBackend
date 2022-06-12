use actix_web::ResponseError;
use std::fmt::Display;

#[derive(Debug)]
pub enum CustomError {
    DieselError(diesel::result::Error),
    FileSystemError(std::io::Error),
    JwtError(jsonwebtoken::errors::Error),
    R2D2Error,
    AlreadyExistsError,
}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "There was an error: {}", self)
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


impl ResponseError for CustomError {}
