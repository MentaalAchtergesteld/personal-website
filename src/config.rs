use std::{env, net::SocketAddr, path::PathBuf, time::Duration};

use url::Url;

pub struct AdminConfig {
    pub path: String,
    pub password_hash: String,
    pub session_ttl: Duration,
    pub secure_cookie: bool,
}

pub struct Config {
    pub server_address: SocketAddr,
    pub database_path: PathBuf,
    pub jellyfin: JellyfinConfig,
    pub admin: Option<AdminConfig>,
}

pub struct JellyfinConfig {
    pub base_url: Url,
    pub api_key: String,
    pub username: String,
    pub user_id: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let server_address = required_var("SERVER_ADDRESS")?
            .parse::<SocketAddr>()
            .map_err(|e| format!("SERVER_ADDRESS must be a valid socket address: {e}"))?;

        let database_path = env::var_os("DATABASE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("guestbook.db"));

        if database_path.as_os_str().is_empty() {
            return Err("DATABASE_PATH cannot be empty".to_string());
        }

        let admin = match env::var("ADMIN_PASSWORD_HASH") {
            Ok(password_hash) if password_hash.trim().is_empty() => {
                return Err("ADMIN_PASSWORD_HASH cannot be empty".to_string());
            }
            Ok(password_hash) => {
                let path = env::var("ADMIN_PATH").unwrap_or_else(|_| "/backstage".to_string());
                validate_admin_path(&path)?;

                let session_ttl_seconds = env::var("ADMIN_SESSION_TTL_SECONDS")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse::<u64>()
                    .map_err(|e| {
                        format!("ADMIN_SESSION_TTL_SECONDS must be a positive integer: {e}")
                    })?;

                if !(60..=604_800).contains(&session_ttl_seconds) {
                    return Err(
                        "ADMIN_SESSION_TTL_SECONDS must be between 60 and 604800 seconds"
                            .to_string(),
                    );
                }

                let secure_cookie = env::var("ADMIN_COOKIE_SECURE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse::<bool>()
                    .map_err(|e| format!("ADMIN_COOKIE_SECURE must be true or false: {e}"))?;

                Some(AdminConfig {
                    path,
                    password_hash,
                    session_ttl: Duration::from_secs(session_ttl_seconds),
                    secure_cookie,
                })
            }
            Err(env::VarError::NotPresent) => None,
            Err(env::VarError::NotUnicode(_)) => {
                return Err("ADMIN_PASSWORD_HASH must contain valid Unicode".to_string());
            }
        };

        let jellyfin_url =
            env::var("JELLYFIN_URL").unwrap_or_else(|_| "https://jellyfin.mentaal.eu/".to_string());
        let base_url = Url::parse(&jellyfin_url)
            .map_err(|e| format!("JELLYFIN_URL must be a valid URL: {e}"))?;
        if !matches!(base_url.scheme(), "http" | "https") {
            return Err("JELLYFIN_URL must use http or https".to_string());
        }

        let username = env::var("JELLYFIN_USERNAME").unwrap_or_else(|_| "mentaal".to_string());
        if username.trim().is_empty() {
            return Err("JELLYFIN_USERNAME cannot be empty".to_string());
        }

        let user_id = optional_var("JELLYFIN_USER_ID")?;

        Ok(Self {
            server_address,
            database_path,
            jellyfin: JellyfinConfig {
                base_url,
                api_key: required_var("JELLYFIN_KEY")?,
                username,
                user_id,
            },
            admin,
        })
    }
}

fn validate_admin_path(path: &str) -> Result<(), String> {
    let is_valid = path.starts_with('/')
        && path.len() > 1
        && !path.ends_with('/')
        && !path.contains(['?', '#'])
        && !path.chars().any(char::is_whitespace)
        && !path.starts_with("/static")
        && !path.starts_with("/comp")
        && !matches!(path, "/home" | "/guestbook" | "/projects" | "/interests");

    if is_valid {
        Ok(())
    } else {
        Err("ADMIN_PATH must be a non-reserved absolute path without a trailing slash".to_string())
    }
}

fn required_var(name: &str) -> Result<String, String> {
    match env::var(name) {
        Ok(value) if value.trim().is_empty() => Err(format!("{name} cannot be empty")),
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Err(format!("{name} is not set")),
        Err(env::VarError::NotUnicode(_)) => Err(format!("{name} must contain valid Unicode")),
    }
}

fn optional_var(name: &str) -> Result<Option<String>, String> {
    match env::var(name) {
        Ok(value) if value.trim().is_empty() => Err(format!("{name} cannot be empty")),
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(format!("{name} must contain valid Unicode")),
    }
}
