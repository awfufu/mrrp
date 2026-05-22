use crate::{
    error::AppError,
    models::rule::RuleName,
    sources::{RuleSource, SourceFuture},
};
use std::fs;

pub struct FileSource {
    template: String,
}

impl FileSource {
    pub fn new(template: String) -> Self {
        Self { template }
    }
}

impl RuleSource for FileSource {
    fn fetch<'a>(&'a self, rule_name: &'a RuleName) -> SourceFuture<'a> {
        Box::pin(async move {
            let path = self.template.replace("{rule}", rule_name.as_str());

            match fs::read_to_string(&path) {
                Ok(content) => Ok(content),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    Err(AppError::RuleNotFound)
                }
                Err(_) => Err(AppError::SourceUnavailable),
            }
        })
    }
}
