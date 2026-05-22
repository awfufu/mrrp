use crate::config::RuleTransformConfig;
use regex::Regex;

pub struct RuleName(String);

impl RuleName {
    pub fn from_owned(name: String) -> Self {
        Self(name)
    }

    pub fn parse(path: &str, transforms: &[RuleTransformConfig]) -> Option<Self> {
        let mut name = path.to_owned();

        for transform in transforms {
            let regex = Regex::new(transform.pattern()).ok()?;
            name = regex
                .replace_all(&name, transform.replace())
                .into_owned();
        }

        if is_valid_name(&name) {
            Some(Self(name))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}
