use crate::{settings, Result};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use directories::{ProjectDirs, UserDirs};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub urls_file: PathBuf, // owned equivalent to Path
    pub postcode:  Option<String>,
}
impl Default for Settings {
    fn default() -> Self {
        let home = get_home_dir().expect("Couldn't compute home dir!");
        let urls_file = home.join("packtrack.urls");
        Self {
            urls_file,
            postcode: None,
        }
    }
}
pub fn reset() -> Result<()> {
    let settings = Settings::default();
    save(&settings)
}
pub fn update(key: String, value: String) -> Result<()> {
    // Serde doesn't support partial deserialization, so to get around this, we
    // insert the user's key/value pair into a mutable settings dict, and
    // deserialize that. This allows us to do a partial update while also
    // validating the new key/value.
    let mut serialized = get_settings_as_dict()?;
    let value = serde_json::to_value(value)?;
    serialized.insert(key, value);

    // Deserialize to Settings for validation
    let serialized = serde_json::to_value(serialized)?;
    let sets: Settings = serde_json::from_value(serialized)?;

    // save Settings
    save(&sets)?;
    Ok(())
}
pub fn print() -> Result<()> {
    let dict = get_settings_as_dict()?;
    for (key, value) in dict.iter() {
        println!("{key}: {value}");
    }
    Ok(())
}
pub fn load() -> Result<Settings> {
    let settings_path = get_settings_path()?;
    if settings_path.exists() {
        let content = fs::read_to_string(settings_path)?;
        let settings: Settings = serde_json::from_str(&content)?;
        Ok(settings)
    } else {
        Ok(Settings::default())
    }
}
pub fn save(settings: &Settings) -> Result<()> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(config_dir)?; // create it if it doesn't exist
    let settings_path = get_settings_path()?;
    let content = serde_json::to_string_pretty(settings)?;
    fs::write(settings_path, content)?;
    Ok(())
}

fn get_config_dir() -> Result<PathBuf> {
    project_dirs().map(|dirs| dirs.config_dir().into())
}

fn get_settings_path() -> Result<PathBuf> {
    get_config_dir().map(|config| config.join("settings.json"))
}

fn get_home_dir() -> Result<PathBuf> {
    UserDirs::new()
        .map(|dirs| dirs.home_dir().into())
        .ok_or("Couldn't compute home dir!".into())
}

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("com", "packtrack", "packtrack")
        .ok_or("Couldn't configure ProjectDirs!".into())
}
fn get_settings_as_dict() -> Result<Map<String, Value>> {
    let sets = load()?;
    let value = serde_json::to_value(sets)?;
    let dict = value
        .as_object()
        .ok_or("Couldn't cast settings to dict!")?;
    Ok(dict.clone())
}
