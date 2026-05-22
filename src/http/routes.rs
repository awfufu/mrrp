use crate::{http::AppState, http::response, services::rule_service::RuleResult};
use axum::{Router, extract::{Path, State}, response::Response, routing::get};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/{path}", get(proxy_rule))
        .with_state(state)
}

async fn proxy_rule(State(state): State<AppState>, Path(path): Path<String>) -> Response {
    match state.rule_service.get_rule(&path).await {
        Ok(RuleResult { body }) => response::text(body),
        Err(error) => response::empty(error.status_code()),
    }
}
