use serde::{Deserialize, Serialize};
use std::env;

// Defaults
const DEFAULT_HOST: &str = "0.0.0.0"; // Default behavior is to listen on all interfaces, since this is expected to be accessed remotely
const DEFAULT_PORT: u16 = 17014;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // Web Server settings
    pub host: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        let mut config = Config::default();

        // Override with environment variables if they exist
        if let Ok(host) = env::var("CUSE_HOST") {
            config.host = host;
        }

        if let Ok(port) = env::var("CUSE_PORT") {
            config.port = port.parse().unwrap_or(config.port);
        }

        config
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.host, DEFAULT_HOST);
    }

    #[test]
    fn test_env_override() {
        env::set_var("CUSE_PORT", "9090");
        env::set_var("CUSE_HOST", "127.0.0.1");

        let config = Config::new();
        assert_eq!(config.port, 9090);
        assert_eq!(config.host, "127.0.0.1");

        // Clean up
        env::remove_var("CUSE_PORT");
        env::remove_var("CUSE_HOST");
    }
}
