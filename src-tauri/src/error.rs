use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Other(String),
    #[error("{0}")]
    LockPoisoned(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),
    #[error(transparent)]
    DieselConnection(#[from] diesel::ConnectionError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    Cdp(#[from] chromiumoxide::error::CdpError),
}

impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

impl From<AppError> for String {
    fn from(value: AppError) -> Self {
        value.to_string()
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}
