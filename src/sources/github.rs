use crate::{error::AppError, models::rule::RuleName};
use axum::http::StatusCode;
use reqwest::Client;

pub struct GithubSource {
    client: Client,
    upstream_base: String,
}

impl GithubSource {
    pub fn new(client: Client, upstream_base: String) -> Self {
        Self {
            client,
            upstream_base,
        }
    }

    pub async fn fetch(&self, rule_name: &RuleName) -> Result<String, AppError> {
        let upstream_url = format!(
            "{}/{name}/{name}.list",
            self.upstream_base,
            name = rule_name.as_str(),
        );

        match self.client.get(upstream_url).send().await {
            Ok(response) if response.status() == StatusCode::OK => response
                .text()
                .await
                .map_err(|_| AppError::UpstreamUnavailable),
            Ok(response) if response.status() == StatusCode::NOT_FOUND => {
                Err(AppError::UpstreamNotFound)
            }
            Ok(_) | Err(_) => Err(AppError::UpstreamUnavailable),
        }
    }
}
