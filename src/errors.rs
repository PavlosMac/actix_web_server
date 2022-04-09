use actix_web::{error::ResponseError, HttpResponse};
use url::ParseError;

#[derive(Fail, Error, Debug)]
pub enum AppError {
    #[fail(display = "Internal Server Error")]
    InternalServerError,

    #[fail(display = "BadRequest: {}", _0)]
    BadRequest(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            AppError::InternalServerError => {
                HttpResponse::InternalServerError().json("Internal Server Error")
            }
            AppError::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
        }
    }
}

impl From<ParseError> for AppError {
    fn from(_: url::ParseError) -> AppError {
        AppError::BadRequest("Invalid value for domain passed".into())
    }
}
