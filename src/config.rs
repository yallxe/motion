#[derive(Debug, Clone)]
pub struct Configuration {
    pub downstreams: Vec<DownstreamConfig>,
    pub bind_address: String,
}

#[derive(Clone, Debug)]
pub struct DownstreamConfig {
    pub address: String,
    pub name: String,
    pub default: bool
}
