use std::sync::{LazyLock, RwLock};

use dotenvy::dotenv;

#[derive(Debug, Clone)]
pub struct Config {
    pub version: String,
    pub api_key: String,
    pub alist_host: String,
    pub alist_token: String,
    pub alist_dir: String,
}

static GLOBAL_CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| {
    RwLock::new(load_config())
});

fn load_config() -> Config {
    // load config from file
    if let Err(e) = dotenv() {
        println!("Error loading .env file: {}", e);
    }

    Config {
        version: std::env::var("VERSION").unwrap_or("0".into()),
        alist_host: std::env::var("ALIST_HOST").unwrap_or("http://localhost:8080".into()),
        alist_token: std::env::var("ALIST_TOKEN").unwrap_or("token".into()),
        alist_dir: std::env::var("ALIST_DIR").unwrap_or("".into()),
        api_key: std::env::var("API_KEY").unwrap_or("".into()),
    }
}

pub fn get_config() -> Config {
    let c = GLOBAL_CONFIG.read().unwrap();
    c.clone()
}

pub fn reload_config() {
    let mut c = GLOBAL_CONFIG.write().unwrap();
    *c = load_config();
}

// test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        load_config();
        println!("{:?}", get_config());
        assert_eq!(get_config().version, "1.0.0");
    }
}