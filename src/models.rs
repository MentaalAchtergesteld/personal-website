use std::{fs, path::Path};

use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Project {
    pub title: String,
    pub description: String,
    pub source_url: String,
    pub deploy_url: Option<String>,
    pub image_url: Option<String>
}

#[derive(Deserialize)]
pub struct ProjectsFile {
    project: Vec<Project>,
}

pub fn load_projects<P: AsRef<Path>>(path: P) -> Result<Vec<Project>, ()> {
    let file_content = fs::read_to_string(path)
        .map_err(|e| eprintln!("ERROR: Couldn't load projects: {e}"))?;
    let data: ProjectsFile = toml::from_str(&file_content)
        .map_err(|e| eprintln!("ERROR: Couldn't parse projects: {e}"))?;

    Ok(data.project)
}

#[derive(Debug)]
pub struct Message {
    pub id: i32,
    pub author: String,
    pub content: String,
    pub timestamp: DateTime<Utc>
}
