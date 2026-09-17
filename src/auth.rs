use std::{
    collections::HashMap,
    fmt::Write,
    sync::Mutex,
    time::{Duration, Instant},
};

use argon2::{
    password_hash::{phc::PasswordHash, PasswordVerifier},
    Argon2,
};

use crate::config::AdminConfig;

struct AdminSession {
    expires_at: Instant,
    csrf_token: String,
}

pub struct AdminAuth {
    config: AdminConfig,
    sessions: Mutex<HashMap<String, AdminSession>>,
}

impl AdminAuth {
    pub fn new(config: AdminConfig) -> Result<Self, String> {
        PasswordHash::new(&config.password_hash)
            .map_err(|e| format!("ADMIN_PASSWORD_HASH is invalid: {e}"))?;

        Ok(Self {
            config,
            sessions: Mutex::new(HashMap::new()),
        })
    }

    pub fn verify_password(&self, password: &str) -> bool {
        let Ok(hash) = PasswordHash::new(&self.config.password_hash) else {
            return false;
        };

        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok()
    }

    pub fn create_session(&self) -> Result<String, getrandom::Error> {
        let token = random_token()?;
        let csrf_token = random_token()?;

        let expires_at = Instant::now()
            .checked_add(self.config.session_ttl)
            .expect("validated session TTL should fit in Instant");

        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(
                token.clone(),
                AdminSession {
                    expires_at,
                    csrf_token,
                },
            );

        Ok(token)
    }

    pub fn session_csrf_token(&self, token: &str) -> Option<String> {
        let now = Instant::now();
        let mut sessions = self
            .sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        sessions.retain(|_, session| session.expires_at > now);
        sessions
            .get(token)
            .map(|session| session.csrf_token.clone())
    }

    pub fn revoke_session(&self, token: &str) {
        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(token);
    }

    pub fn session_ttl(&self) -> Duration {
        self.config.session_ttl
    }

    pub fn path(&self) -> &str {
        &self.config.path
    }

    pub fn is_login_path(&self, path: &str) -> bool {
        path.strip_prefix(&self.config.path) == Some("/login")
    }

    pub fn is_logout_path(&self, path: &str) -> bool {
        path.strip_prefix(&self.config.path) == Some("/logout")
    }

    pub fn uses_secure_cookie(&self) -> bool {
        self.config.secure_cookie
    }
}

fn random_token() -> Result<String, getrandom::Error> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes)?;

    let mut token = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut token, "{byte:02x}").expect("writing to a String cannot fail");
    }

    Ok(token)
}
