use serde::Deserialize;
use std::{
    env, fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};

const DEFAULT_CONFIG_PATH: &str = "config.yml";

pub struct Config {
    server: ServerConfig,
    rule_transforms: Vec<RuleTransformConfig>,
    upstreams: Vec<UpstreamConfig>,
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

#[derive(Clone)]
pub enum UpstreamConfig {
    Url {
        template: String,
        remove_comments: bool,
    },
    File {
        template: String,
        remove_comments: bool,
    },
}

#[derive(Default, Deserialize)]
struct FileConfig {
    #[serde(rename = "server-ip")]
    server_ip: Option<IpAddr>,
    #[serde(rename = "server-port")]
    server_port: Option<u16>,
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

impl TryFrom<FileUpstreamConfig> for UpstreamConfig {
    type Error = String;

    fn try_from(value: FileUpstreamConfig) -> Result<Self, Self::Error> {
        match value.kind.as_str() {
            "url" => Ok(Self::Url {
                template: value.template,
                remove_comments: value.remove_comments.unwrap_or(true),
            }),
            "file" => Ok(Self::File {
                template: value.template,
                remove_comments: value.remove_comments.unwrap_or(true),
            }),
            other => Err(format!("unsupported upstream type: {other}")),
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
