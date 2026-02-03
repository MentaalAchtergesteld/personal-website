use std::{sync::{Arc, RwLock}, time::{Duration, Instant}};

struct CacheState<T> {
    data: Option<Arc<T>>,
    last_updated: Instant,
}

impl<T> CacheState<T> {
    pub fn is_valid(&self, ttl: Duration) -> bool {
        self.data.is_some() && self.last_updated.elapsed() < ttl
    }
}

pub struct Cache<T> {
    state: Arc<RwLock<CacheState<T>>>,
    ttl: Duration,
}

impl<T: Send + Sync + 'static> Cache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            state: Arc::new(RwLock::new(CacheState {
                data: None,
                last_updated: Instant::now() - ttl - ttl,
            })),
            ttl,
        }
    }

    pub fn get_or_update<F>(&self, fetcher: F) -> Option<Arc<T>>
    where F: FnOnce() -> Option<T>,
    {
        if let Ok(guard) = self.state.read() {
            if guard.is_valid(self.ttl) { return guard.data.clone() }
        }

        let mut guard = self.state.write().unwrap();

        if guard.is_valid(self.ttl) { return guard.data.clone() }

        if let Some(fresh_data) = fetcher() {
            let arc_data = Arc::new(fresh_data);
            guard.data = Some(arc_data.clone());
            guard.last_updated = Instant::now();
            return Some(arc_data);
        }

        guard.data.clone()
    }
}
