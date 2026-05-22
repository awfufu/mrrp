use crate::{
    config::{Config, UpstreamConfig},
    error::AppError,
    models::rule::RuleName,
    services::filter::filter_rule_lines,
    sources::{RuleSource, filesystem::FileSource, github::UrlSource},
};
use reqwest::{Client, Proxy, header::{HeaderMap, HeaderName, HeaderValue}};
use std::time::Duration;

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
                    proxy,
                    timeout_ms,
                    headers,
                } => {
                    sources.push(ConfiguredSource {
                        source: Box::new(UrlSource::new(
                            build_url_client(
                                &client,
                                proxy.as_deref(),
                                *timeout_ms,
                                headers,
                            )?,
                            template.clone(),
                        )),
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

fn build_url_client(
    base_client: &Client,
    proxy: Option<&str>,
    timeout_ms: Option<u64>,
    headers: &[String],
) -> Result<Client, String> {
    if proxy.is_none() && timeout_ms.is_none() && headers.is_empty() {
        return Ok(base_client.clone());
    }

    let mut builder = Client::builder();

    if let Some(proxy) = proxy {
        let reqwest_proxy =
            Proxy::all(proxy).map_err(|error| format!("invalid upstream proxy {proxy}: {error}"))?;
        builder = builder.proxy(reqwest_proxy);
    }

    if let Some(timeout_ms) = timeout_ms {
        builder = builder.timeout(Duration::from_millis(timeout_ms));
    }

    if !headers.is_empty() {
        builder = builder.default_headers(build_headers(headers)?);
    }

    builder
        .build()
        .map_err(|error| format!("failed to build url upstream client: {error}"))
}

fn build_headers(headers: &[String]) -> Result<HeaderMap, String> {
    let mut header_map = HeaderMap::new();

    for header in headers {
        let Some((name, value)) = header.split_once(':') else {
            return Err(format!("invalid header format: {header}"));
        };

        let header_name = HeaderName::from_bytes(name.trim().as_bytes())
            .map_err(|error| format!("invalid header name in {header}: {error}"))?;
        let header_value = HeaderValue::from_str(value.trim())
            .map_err(|error| format!("invalid header value in {header}: {error}"))?;

        header_map.append(header_name, header_value);
    }

    Ok(header_map)
}
