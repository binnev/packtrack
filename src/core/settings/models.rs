use crate::cache::get_cache_dir;
use crate::{Result, utils::get_home_dir};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub urls_file:         PathBuf, // owned equivalent to Path
    pub postcode:          Option<String>,
    pub language:          Option<String>,
    pub cache_file:        PathBuf,
    /// Maximum age (in seconds) for cache entries to be reused.
    pub cache_seconds:     usize,
    /// Maximum number of entries to cache (per URL)
    pub cache_max_entries: usize,
}
impl Settings {
    /// Handle updating arbitrary key/value pairs. These could come from the CLI
    /// or API query parameters, for example.
    pub fn update(
        &mut self,
        key: &str,
        value: impl Into<String>,
    ) -> Result<()> {
        let value: String = value.into();
        match key {
            "urls_file" => {
                let path: PathBuf = value.into();
                if !path.try_exists()? {
                    return Err(
                        format!("urls_file doesn't exist: {path:?}").into()
                    );
                }
                self.urls_file = path;
            }
            "postcode" => self.postcode = Some(value),
            "language" => self.language = Some(value),
            "cache_seconds" => self.cache_seconds = value.parse()?,
            "cache_max_entries" => self.cache_max_entries = value.parse()?,
            _ => return Err(format!("Invalid setting key: {key}").into()),
        }
        Ok(())
    }
    pub fn default() -> Result<Self> {
        let home = get_home_dir().expect("Couldn't compute home dir!");
        let urls_file = home.join("packtrack.urls");
        Ok(Self {
            urls_file,
            postcode: None,
            language: None,
            cache_file: get_cache_dir()?.join("packtrack-cache.json"),
            cache_seconds: 30,
            cache_max_entries: 10,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_update_invalid_key() -> Result<()> {
        let result = Settings::default()?.update("Foo", "Bar");
        assert_eq!(result.err().unwrap(), "Invalid setting key: Foo".into());
        Ok(())
    }

    #[test]
    fn test_settings_update_string() -> Result<()> {
        let mut settings = Settings::default()?;
        settings.update("postcode", "1234AB")?;
        assert_eq!(settings.postcode.unwrap(), "1234AB");
        Ok(())
    }

    #[test]
    fn test_settings_update_int() -> Result<()> {
        let mut settings = Settings::default()?;
        settings.update("cache_seconds", "30")?;
        assert_eq!(settings.cache_seconds, 30);

        let result = Settings::default()?.update("cache_seconds", "thirty");
        assert!(
            format!("{:?}", result.err().unwrap()).contains("ParseIntError")
        );
        Ok(())
    }

    #[test]
    fn test_settings_update_path() -> Result<()> {
        let mut settings = Settings::default()?;
        settings.update("urls_file", ".")?;
        assert_eq!(format!("{:?}", settings.urls_file), "\".\"");

        let result = Settings::default()?.update("urls_file", "xxxxx");
        let err = result.err().unwrap().to_string();
        assert!(err.contains("urls_file doesn't exist:"));
        assert!(err.contains("xxxxx"));
        Ok(())
    }
}
