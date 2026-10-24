use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Internal error")]
    Internal,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Not found")]
    NotFound,
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Validation: {0}")]
    Validation(String),
}

pub type AppResult<T> = Result<T, AppError>;