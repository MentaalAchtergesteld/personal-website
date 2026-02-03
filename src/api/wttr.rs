use chrono::{DateTime, TimeDelta, Utc};

const WTTR_URL: &str = "http://wttr.in/Eindhoven?format=2";

pub struct WttrApi {
    cached_weather: String,
    last_update: DateTime<Utc>,
    cache_interval: TimeDelta
}

impl WttrApi {
    pub fn new(cache_interval: TimeDelta) -> Self {
        Self {
            cached_weather: "Loading...".into(),
            last_update: DateTime::from_timestamp_millis(0).unwrap(),
            cache_interval
        }
    }

    fn update_cache(&mut self) {
        match ureq::get(WTTR_URL).call() {
            Ok(res) => {
                self.cached_weather = res.into_string()
                    .unwrap_or("Couldn't load weather.".into());
            },
            Err(e) => eprintln!("Failed to update weather: {e}"),
        }
    }

    pub fn get_weather(&mut self) -> &str {
        let now = Utc::now(); 
        let since_last_update = now - self.last_update;
        if since_last_update > self.cache_interval {
            self.last_update = now;
            self.update_cache();
        }

        &self.cached_weather
    }
}
