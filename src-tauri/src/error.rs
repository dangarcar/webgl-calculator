use serde::Serialize;

pub type Result<T> = core::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error, Clone)]
pub enum AppError {
    #[error("Couldn't parse this latex string: {0}")]
    ParseError(String),
    #[error("Couldn't resolve this math equation: {0}")]
    MathError(String),
    #[error("Empty error, there's nothing there")]
    EmptyError,
    #[error("{0}")]
    IoError(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(self.to_string().as_str())
    }
}