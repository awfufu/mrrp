use crate::{
    config::Config,
    http::{self, AppState},
    services::rule_service::RuleService,
    sources::github::GithubSource,
};
use reqwest::Client;
use std::sync::Arc;

pub async fn run() {
    let config = Config::default();
    let client = Client::new();
    let source = GithubSource::new(client, config.upstream_base.to_owned());
    let rule_service = Arc::new(RuleService::new(source));
    let state = AppState::new(rule_service);
    let app = http::router(state);

    let listener = tokio::net::TcpListener::bind(config.listen_addr)
        .await
        .expect("failed to bind server socket");

    println!("listening on http://{}", config.listen_addr);

    axum::serve(listener, app)
        .await
        .expect("server exited unexpectedly");
}
