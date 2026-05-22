pub mod response;
pub mod routes;

use crate::services::rule_service::RuleService;
use axum::Router;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub rule_service: Arc<RuleService>,
}

impl AppState {
    pub fn new(rule_service: Arc<RuleService>) -> Self {
        Self { rule_service }
    }
}

pub fn router(state: AppState) -> Router {
    routes::router(state)
}
