#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config could not be read: {0}")]
    Read(#[from] std::io::Error),
    #[error("config could not be parsed: {0}")]
    Parse(#[from] serde_json::Error),
}
