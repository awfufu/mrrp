pub mod chain;
pub mod filesystem;
pub mod github;

use crate::{error::AppError, models::rule::RuleName};
use std::{future::Future, pin::Pin};

pub type SourceFuture<'a> = Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'a>>;

pub trait RuleSource: Send + Sync {
    fn fetch<'a>(&'a self, rule_name: &'a RuleName) -> SourceFuture<'a>;
}
