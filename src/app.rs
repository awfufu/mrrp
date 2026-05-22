use crate::{
    config::Config,
    http::{self, AppState},
    services::rule_service::RuleService,
    sources::chain::SourceChain,
};
use reqwest::Client;
use std::sync::Arc;

pub async fn run() {
    let config = Config::load().unwrap_or_else(|error| panic!("{error}"));
    let client = Client::new();
    let source_chain = SourceChain::from_config(&config, client)
        .unwrap_or_else(|error| panic!("{error}"));
    let rule_service = Arc::new(RuleService::new(
        config.rule_transforms().to_vec(),
        source_chain,
    ));
    let state = AppState::new(rule_service);
    let app = http::router(state);

    let listener = tokio::net::TcpListener::bind(config.listen_addr())
        .await
        .expect("failed to bind server socket");

    println!("listening on http://{}", config.listen_addr());

    axum::serve(listener, app)
        .await
        .expect("server exited unexpectedly");
}
