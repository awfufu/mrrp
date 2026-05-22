use crate::{
    config::RuleTransformConfig,
    error::AppError,
    models::rule::RuleName,
    sources::chain::SourceChain,
};

pub struct RuleService {
    rule_transforms: Vec<RuleTransformConfig>,
    source_chain: SourceChain,
}

pub struct RuleResult {
    pub body: String,
}

impl RuleService {
    pub fn new(rule_transforms: Vec<RuleTransformConfig>, source_chain: SourceChain) -> Self {
        Self {
            rule_transforms,
            source_chain,
        }
    }

    pub async fn get_rule(&self, path: &str) -> Result<RuleResult, AppError> {
        let rule_name = RuleName::parse(path, &self.rule_transforms)
            .ok_or(AppError::InvalidRuleName)?;
        let body = self.source_chain.fetch(&rule_name).await?;

        Ok(RuleResult { body })
    }
}
