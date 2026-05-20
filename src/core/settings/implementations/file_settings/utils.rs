use crate::{Result, utils::project_dirs};
use std::path::PathBuf;

fn get_config_dir() -> Result<PathBuf> {
    project_dirs().map(|dirs| dirs.config_dir().into())
}

pub fn get_settings_file() -> Result<PathBuf> {
    get_config_dir().map(|config| config.join("settings.json"))
}
