use packtrack::{
    Result,
    utils::{get_home_dir, load_json, project_dirs, save_json},
};
use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub urls_file:         PathBuf, // owned equivalent to Path
    pub postcode:          Option<String>,
    pub language:          Option<String>,
    /// Maximum age (in seconds) for cache entries to be reused.
    pub cache_seconds:     usize,
    /// Maximum number of entries to cache (per URL)
    pub cache_max_entries: usize,
}
impl Settings {
    /// Handle updating arbitrary key/value pairs. These could come from the CLI
    /// or API query parameters, for example.
    fn update(mut self, key: &str, value: impl Into<String>) -> Result<Self> {
        let value: String = value.into();
        match key {
            // TODO: should check that this is a valid path
            "urls_file" => self.urls_file = value.into(),
            "postcode" => self.postcode = Some(value),
            "language" => self.language = Some(value),
            "cache_seconds" => self.cache_seconds = value.parse()?,
            "cache_max_entries" => self.cache_max_entries = value.parse()?,
            _ => return Err(format!("Invalid setting key: {key}").into()),
        }
        Ok(self)
    }
}
impl Default for Settings {
    fn default() -> Self {
        let home = get_home_dir().expect("Couldn't compute home dir!");
        let urls_file = home.join("packtrack.urls");
        Self {
            urls_file,
            postcode: None,
            language: None,
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
pub fn update(key: &str, value: String) -> Result<()> {
    let mut sets = load()?;
    sets = sets.update(key, value)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_update_invalid_key() {
        let result = Settings::default().update("Foo", "Bar");
        assert_eq!(result.err().unwrap(), "Invalid setting key: Foo".into());
    }

    #[test]
    fn test_settings_update_string() -> Result<()> {
        let settings = Settings::default().update("postcode", "1234AB")?;
        assert_eq!(settings.postcode.unwrap(), "1234AB");
        Ok(())
    }

    #[test]
    fn test_settings_update_int() -> Result<()> {
        let settings = Settings::default().update("cache_seconds", "30")?;
        assert_eq!(settings.cache_seconds, 30);

        let result = Settings::default().update("cache_seconds", "thirty");
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("ParseIntError")
        );
        Ok(())
    }
}
