use std::{collections::HashMap, net::IpAddr, time::{Duration, Instant}};

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
            Some(&last) if now.duration_since(last) < self.cooldown => false,
            _ => {
                self.last_request.insert(ip, now);
                true
            }
        }
    }
}
