use crate::{
    settings,
    utils::{get_home_dir, load_json, project_dirs, save_json},
    Result,
};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use directories::{ProjectDirs, UserDirs};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub urls_file:         PathBuf, // owned equivalent to Path
    pub postcode:          Option<String>,
    pub use_cache:         bool,
    /// Maximum age (in seconds) for cache entries to be reused.
    pub cache_seconds:     u64,
    /// Maximum number of entries to cache (per URL)
    pub cache_max_entries: usize,
}
impl Default for Settings {
    fn default() -> Self {
        let home = get_home_dir().expect("Couldn't compute home dir!");
        let urls_file = home.join("packtrack.urls");
        Self {
            urls_file,
            postcode: None,
            use_cache: true,
            cache_seconds: 30,
            cache_max_entries: 10,
        }
    }
}

pub fn reset() -> Result<()> {
    let settings = Settings::default();
    save(&settings)
}
/// Update a key/value pair in the settings, and save them to file.
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
    // Load settings from file (these may be incomplete, so we don't cast them
    // to Settings just yet)
    let from_file: HashMap<String, Value> = load_json(&get_settings_path()?)?;
    // Use defaults to supply any missing values
    let mut defaults = serde_json::to_value(Settings::default())?
        .as_object()
        .ok_or("Couldn't cast default Settings to HashMap?!")?
        .clone();
    // Merge the two, with the values from file taking priority. Now we should
    // have a complete settings dict.
    defaults.extend(from_file);
    // deserialize to Settings
    let sets: Settings = serde_json::from_value(Value::Object(defaults))?;
    Ok(sets)
}
pub fn save(settings: &Settings) -> Result<()> {
    save_json(&get_settings_path()?, settings)
}

fn get_config_dir() -> Result<PathBuf> {
    project_dirs().map(|dirs| dirs.config_dir().into())
}

fn get_settings_path() -> Result<PathBuf> {
    get_config_dir().map(|config| config.join("settings.json"))
}

fn get_settings_as_dict() -> Result<Map<String, Value>> {
    let sets = load()?;
    let value = serde_json::to_value(sets)?;
    let dict = value
        .as_object()
        .ok_or("Couldn't cast settings to dict!")?;
    Ok(dict.clone())
}
