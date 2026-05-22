pub struct RuleName(String);

impl RuleName {
    pub fn parse(path: &str) -> Option<Self> {
        let name = path.strip_suffix(".list").unwrap_or(path);

        if is_valid_name(name) {
            Some(Self(name.to_owned()))
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
