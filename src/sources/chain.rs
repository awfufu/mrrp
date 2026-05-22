use crate::{
    config::{Config, UpstreamConfig},
    error::AppError,
    models::rule::RuleName,
    services::filter::filter_rule_lines,
    sources::{RuleSource, filesystem::FileSource, github::UrlSource},
};
use reqwest::Client;

pub struct SourceChain {
    sources: Vec<ConfiguredSource>,
}

struct ConfiguredSource {
    source: Box<dyn RuleSource>,
    remove_comments: bool,
}

impl SourceChain {
    pub fn from_config(config: &Config, client: Client) -> Result<Self, String> {
        let mut sources: Vec<ConfiguredSource> = Vec::new();

        for upstream in config.upstreams() {
            match upstream {
                UpstreamConfig::Url {
                    template,
                    remove_comments,
                } => {
                    sources.push(ConfiguredSource {
                        source: Box::new(UrlSource::new(client.clone(), template.clone())),
                        remove_comments: *remove_comments,
                    });
                }
                UpstreamConfig::File {
                    template,
                    remove_comments,
                } => {
                    sources.push(ConfiguredSource {
                        source: Box::new(FileSource::new(template.clone())),
                        remove_comments: *remove_comments,
                    });
                }
            }
        }

        if sources.is_empty() {
            return Err("config must define at least one upstream".to_owned());
        }

        Ok(Self { sources })
    }

    pub async fn fetch(&self, rule_name: &RuleName) -> Result<String, AppError> {
        let mut saw_unavailable = false;

        for source in &self.sources {
            match source.source.fetch(rule_name).await {
                Ok(content) => {
                    return Ok(if source.remove_comments {
                        filter_rule_lines(&content)
                    } else {
                        content
                    });
                }
                Err(AppError::RuleNotFound) => continue,
                Err(AppError::SourceUnavailable) => saw_unavailable = true,
                Err(AppError::InvalidRuleName) => return Err(AppError::InvalidRuleName),
            }
        }

        if saw_unavailable {
            Err(AppError::SourceUnavailable)
        } else {
            Err(AppError::RuleNotFound)
        }
    }
}
