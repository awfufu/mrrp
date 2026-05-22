use serde::Deserialize;
use std::{
    env, fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    time::Duration,
};

const DEFAULT_CONFIG_PATH: &str = "config.yml";
const DEFAULT_CACHE: &str = "30m";

pub struct Config {
    server: ServerConfig,
    rule_transforms: Vec<RuleTransformConfig>,
    upstreams: Vec<UpstreamConfig>,
    upstream_mode: UpstreamMode,
}

pub struct ServerConfig {
    listen_ip: IpAddr,
    listen_port: u16,
}

#[derive(Clone)]
pub struct RuleTransformConfig {
    pattern: String,
    replace: String,
}

#[derive(Clone, Copy)]
pub enum UpstreamMode {
    Race,
    Sequential,
}

#[derive(Clone)]
pub enum UpstreamConfig {
    Url {
        template: String,
        remove_comments: bool,
        proxy: Option<String>,
        timeout_ms: Option<u64>,
        headers: Vec<String>,
        cache_ttl: Duration,
    },
    File {
        template: String,
        remove_comments: bool,
        cache_ttl: Duration,
    },
}

#[derive(Default, Deserialize)]
struct FileConfig {
    #[serde(rename = "server-ip")]
    server_ip: Option<IpAddr>,
    #[serde(rename = "server-port")]
    server_port: Option<u16>,
    #[serde(rename = "upstream-mode")]
    upstream_mode: Option<String>,
    #[serde(rename = "rule-transforms")]
    rule_transforms: Option<Vec<FileRuleTransformConfig>>,
    upstreams: Option<Vec<FileUpstreamConfig>>,
}

#[derive(Deserialize)]
struct FileRuleTransformConfig {
    pattern: String,
    replace: String,
}

#[derive(Deserialize)]
struct FileUpstreamConfig {
    #[serde(rename = "type")]
    kind: String,
    template: String,
    #[serde(rename = "remove-comments")]
    remove_comments: Option<bool>,
    proxy: Option<String>,
    #[serde(rename = "timeout-ms")]
    timeout_ms: Option<u64>,
    headers: Option<HeaderConfig>,
    cache: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum HeaderConfig {
    Single(String),
    Multiple(Vec<String>),
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = parse_config_path(env::args().skip(1))?;

        match config_path {
            Some(path) => Self::from_path(&path),
            None => Ok(Self::default()),
        }
    }

    pub fn listen_addr(&self) -> SocketAddr {
        SocketAddr::from((self.server.listen_ip, self.server.listen_port))
    }

    pub fn rule_transforms(&self) -> &[RuleTransformConfig] {
        &self.rule_transforms
    }

    pub fn upstreams(&self) -> &[UpstreamConfig] {
        &self.upstreams
    }

    pub fn upstream_mode(&self) -> UpstreamMode {
        self.upstream_mode
    }

    fn from_path(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|error| format!("failed to read config file {}: {error}", path.display()))?;
        let file_config: FileConfig = serde_yaml::from_str(&content)
            .map_err(|error| format!("failed to parse config file {}: {error}", path.display()))?;

        Ok(Self::from_file_config(file_config))
    }

    fn from_file_config(file_config: FileConfig) -> Self {
        let mut config = Self::default();

        if let Some(ip) = file_config.server_ip {
            config.server.listen_ip = ip;
        }

        if let Some(port) = file_config.server_port {
            config.server.listen_port = port;
        }

        if let Some(rule_transforms) = file_config.rule_transforms {
            config.rule_transforms = rule_transforms
                .into_iter()
                .map(|transform| RuleTransformConfig {
                    pattern: transform.pattern,
                    replace: transform.replace,
                })
                .collect();
        }

        if let Some(upstream_mode) = file_config.upstream_mode {
            config.upstream_mode = UpstreamMode::try_from(upstream_mode.as_str())
                .unwrap_or_else(|error| panic!("{error}"));
        }

        if let Some(upstreams) = file_config.upstreams {
            config.upstreams = upstreams
                .into_iter()
                .map(UpstreamConfig::try_from)
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|error| panic!("{error}"));
        }

        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            rule_transforms: Vec::new(),
            upstreams: Vec::new(),
            upstream_mode: UpstreamMode::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            listen_port: 8044,
        }
    }
}

impl RuleTransformConfig {
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn replace(&self) -> &str {
        &self.replace
    }
}

impl Default for UpstreamMode {
    fn default() -> Self {
        Self::Race
    }
}

impl TryFrom<&str> for UpstreamMode {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "race" => Ok(Self::Race),
            "sequential" => Ok(Self::Sequential),
            other => Err(format!("unsupported upstream mode: {other}")),
        }
    }
}

impl TryFrom<FileUpstreamConfig> for UpstreamConfig {
    type Error = String;

    fn try_from(value: FileUpstreamConfig) -> Result<Self, Self::Error> {
        match value.kind.as_str() {
            "url" => Ok(Self::Url {
                template: value.template,
                remove_comments: value.remove_comments.unwrap_or(true),
                proxy: value.proxy,
                timeout_ms: value.timeout_ms,
                headers: value.headers.map(HeaderConfig::into_vec).unwrap_or_default(),
                cache_ttl: parse_cache_ttl(value.cache.as_deref().unwrap_or(DEFAULT_CACHE))?,
            }),
            "file" => {
                if value.proxy.is_some() {
                    return Err("proxy is only supported for url upstreams".to_owned());
                }

                if value.timeout_ms.is_some() {
                    return Err("timeout-ms is only supported for url upstreams".to_owned());
                }

                if value.headers.is_some() {
                    return Err("headers are only supported for url upstreams".to_owned());
                }

                Ok(Self::File {
                    template: value.template,
                    remove_comments: value.remove_comments.unwrap_or(true),
                    cache_ttl: parse_cache_ttl(value.cache.as_deref().unwrap_or(DEFAULT_CACHE))?,
                })
            }
            other => Err(format!("unsupported upstream type: {other}")),
        }
    }
}

impl HeaderConfig {
    fn into_vec(self) -> Vec<String> {
        match self {
            Self::Single(value) => vec![value],
            Self::Multiple(values) => values,
        }
    }
}

fn parse_config_path(args: impl Iterator<Item = String>) -> Result<Option<PathBuf>, String> {
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-f" | "--config" => {
                let path = args
                    .next()
                    .ok_or_else(|| format!("missing config file path after {arg}"))?;
                return Ok(Some(PathBuf::from(path)));
            }
            _ => {
                return Err(format!("unsupported argument: {arg}"));
            }
        }
    }

    let default_path = PathBuf::from(DEFAULT_CONFIG_PATH);

    if default_path.exists() {
        Ok(Some(default_path))
    } else {
        Ok(None)
    }
}

fn parse_cache_ttl(value: &str) -> Result<Duration, String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("cache cannot be empty".to_owned());
    }

    let digits_len = trimmed.bytes().take_while(|byte| byte.is_ascii_digit()).count();

    if digits_len == 0 {
        return Err(format!("invalid cache duration: {value}"));
    }

    let amount = trimmed[..digits_len]
        .parse::<u64>()
        .map_err(|error| format!("invalid cache duration {value}: {error}"))?;
    let unit = trimmed[digits_len..].trim();
    let millis = match unit {
        "" => amount,
        "s" => amount.saturating_mul(1_000),
        "m" => amount.saturating_mul(60_000),
        "h" => amount.saturating_mul(3_600_000),
        "d" => amount.saturating_mul(86_400_000),
        _ => return Err(format!("unsupported cache duration unit in {value}")),
    };

    Ok(Duration::from_millis(millis))
}
