use axum::http::StatusCode;

pub enum AppError {
    InvalidRuleName,
    RuleNotFound,
    SourceUnavailable,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRuleName | Self::RuleNotFound => StatusCode::NOT_FOUND,
            Self::SourceUnavailable => StatusCode::BAD_GATEWAY,
        }
    }
}
