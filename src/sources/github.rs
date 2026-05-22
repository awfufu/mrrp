use crate::{error::AppError, models::rule::RuleName, sources::{RuleSource, SourceFuture}};
use axum::http::StatusCode;
use reqwest::Client;

pub struct UrlSource {
    client: Client,
    template: String,
}

impl UrlSource {
    pub fn new(client: Client, template: String) -> Self {
        Self {
            client,
            template,
        }
    }
}

impl RuleSource for UrlSource {
    fn fetch<'a>(&'a self, rule_name: &'a RuleName) -> SourceFuture<'a> {
        Box::pin(async move {
            let upstream_url = self.template.replace("{rule}", rule_name.as_str());

            match self.client.get(upstream_url).send().await {
                Ok(response) if response.status() == StatusCode::OK => response
                    .text()
                    .await
                    .map_err(|_| AppError::SourceUnavailable),
                Ok(response) if response.status() == StatusCode::NOT_FOUND => {
                    Err(AppError::RuleNotFound)
                }
                Ok(_) | Err(_) => Err(AppError::SourceUnavailable),
            }
        })
    }
}
