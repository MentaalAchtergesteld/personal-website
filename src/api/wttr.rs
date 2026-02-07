const WTTR_URL: &str = "http://wttr.in/Eindhoven?format=2";

pub struct WttrApi {
    agent: ureq::Agent,
}

impl WttrApi {
    pub fn new() -> Self {
        let agent = ureq::AgentBuilder::new()
            .build();

        Self { agent }
    }
    pub fn get_weather(&self) -> String {
        match self.agent.get(WTTR_URL).call() {
            Ok(res) => {
                res.into_string().unwrap_or("Couldn't get weather".to_string())
            },
            Err(_) => "Wttr Timeout".to_string()
        }
    }
}
