use std::fs;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Project {
    pub title: String,
    pub source_url: String,
    pub deploy_url: Option<String>,
    pub description: String,
    pub image_url: Option<String>
}

pub fn load_projects(path: &str) -> Result<Vec<Project>, ()> {
    #[derive(Deserialize)]
    struct Wrapper { pub project: Vec<Project> }

    let toml_str = fs::read_to_string(path)
        .map_err(|e| eprintln!("ERROR: couldn't read projects file: {e}"))?;

    let wrapper: Wrapper = toml::from_str(&toml_str)
        .map_err(|e| eprintln!("ERROR: couldn't parse projects file: {e}"))?;

    Ok(wrapper.project)
}
