use std::net::SocketAddr;

pub struct Config {
    pub listen_addr: SocketAddr,
    pub upstream_base: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: SocketAddr::from(([0, 0, 0, 0], 8044)),
            upstream_base:
                "https://raw.githubusercontent.com/blackmatrix7/ios_rule_script/refs/heads/master/rule/Clash",
        }
    }
}
