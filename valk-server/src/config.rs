use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

// Defaults
const DEFAULT_HOST: &str = "0.0.0.0"; // Default behavior is to listen on all interfaces, since this is expected to be accessed remotely
const DEFAULT_PORT: u16 = 8255;
const DEFAULT_SESSION_DURATION: u64 = 15 * 60; // 15 minutes

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // Web Server settings
    pub host: String,
    pub port: u16,
    pub session_duration: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
            session_duration: Duration::from_secs(DEFAULT_SESSION_DURATION),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        let mut config = Config::default();

        if let Ok(host) = env::var("VALK_HOST") {
            config.host = host;
        }

        if let Ok(port) = env::var("VALK_PORT") {
            config.port = port.parse().unwrap_or(config.port);
        }

        if let Ok(duration_secs) = env::var("VALK_SESSION_DURATION") {
            if let Ok(secs) = duration_secs.parse::<u64>() {
                config.session_duration = Duration::from_secs(secs);
            }
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
        assert_eq!(
            config.session_duration,
            Duration::from_secs(DEFAULT_SESSION_DURATION)
        );
    }

    #[test]
    fn test_env_override() {
        env::set_var("VALK_PORT", "9090");
        env::set_var("VALK_HOST", "127.0.0.1");
        env::set_var("VALK_SESSION_DURATION", "30");

        let config = Config::new();
        assert_eq!(config.port, 9090);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.session_duration, Duration::from_secs(30 * 60));

        // Clean up
        env::remove_var("VALK_PORT");
        env::remove_var("VALK_HOST");
        env::remove_var("VALK_SESSION_DURATION");
    }
}
