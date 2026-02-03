const WTTR_URL: &str = "http://wttr.in/Eindhoven?format=2";

pub struct WttrApi {}

impl WttrApi {
    pub fn get_weather(&self) -> String {
        let res = ureq::get(WTTR_URL).call();
        match res {
            Ok(res) => res.into_string().ok(),
            Err(e) => {
                eprintln!("Failed to update weather: {e}");
                None
            }
        }.unwrap_or("Couldn't get weather.".to_string())
    }
}
