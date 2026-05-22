use axum::http::StatusCode;

pub enum AppError {
    InvalidRuleName,
    UpstreamNotFound,
    UpstreamUnavailable,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRuleName | Self::UpstreamNotFound => StatusCode::NOT_FOUND,
            Self::UpstreamUnavailable => StatusCode::BAD_GATEWAY,
        }
    }
}
