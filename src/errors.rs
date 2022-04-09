use actix_web::{error::ResponseError, HttpResponse};
use derive_more::Display;
// use url::ParseError;

#[derive(Display, Debug)]
pub enum AppError {
    #[display(fmt = "Internal Server Error: {}", _0)]
    InternalServerError(String),

    #[display(fmt = "BadRequest: {}", _0)]
    BadRequest(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            AppError::InternalServerError(ref message) => HttpResponse::InternalServerError()
                .json(format!("Internal Server Error: {}", message)),
            AppError::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
        }
    }
}

impl From<url::ParseError> for AppError {
    fn from(_: url::ParseError) -> AppError {
        AppError::BadRequest("Invalid value for domain".into())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}

impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}
