use std::{collections::HashMap, net::IpAddr, time::{Duration, Instant}};

use tiny_http::Request;

pub fn get_client_ip(request: &Request) -> Option<IpAddr> {
    if let Some(ip) = request.headers().iter()
        .find(|h| h.field.equiv("CF-Connecting-IP"))
        .and_then(|h| h.value.as_str().parse::<IpAddr>().ok()) 
    {
        return Some(ip);
    }
    request.headers().iter()
        .find(|h| h.field.equiv("X-Forwarded-For"))
        .and_then(|h| h.value.as_str().split(',').next())
        .and_then(|ip_str| ip_str.trim().parse::<IpAddr>().ok())
        .or_else(|| request.remote_addr().map(|addr| addr.ip()))
}

pub struct RateLimiter {
    last_request: HashMap<IpAddr, Instant>,
    cooldown: Duration
}

impl RateLimiter {
    pub fn new(cooldown: Duration) -> Self {
        RateLimiter {
            last_request: HashMap::new(),
            cooldown
        }
    }

    pub fn is_allowed(&mut self, ip: IpAddr) -> bool {
        let now = Instant::now();

        match self.last_request.get(&ip) {
            Some(&last) => {
                let is_allowed = !(now.duration_since(last) < self.cooldown);
                self.last_request.insert(ip, now);
                return is_allowed;
            }
            _ => {
                self.last_request.insert(ip, now);
                true
            }
        }
    }
}
