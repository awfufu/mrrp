use crate::{
    error::AppError,
    models::rule::RuleName,
    services::filter::filter_rule_lines,
    sources::github::GithubSource,
};

pub struct RuleService {
    github_source: GithubSource,
}

pub struct RuleResult {
    pub body: String,
}

impl RuleService {
    pub fn new(github_source: GithubSource) -> Self {
        Self { github_source }
    }

    pub async fn get_rule(&self, path: &str) -> Result<RuleResult, AppError> {
        let rule_name = RuleName::parse(path).ok_or(AppError::InvalidRuleName)?;
        let body = self.github_source.fetch(&rule_name).await?;

        Ok(RuleResult {
            body: filter_rule_lines(&body),
        })
    }
}
