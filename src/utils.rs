use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::Result;
use directories::{ProjectDirs, UserDirs};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("com", "packtrack", "packtrack")
        .ok_or("Couldn't configure ProjectDirs!".into())
}

pub fn get_home_dir() -> Result<PathBuf> {
    UserDirs::new()
        .map(|dirs| dirs.home_dir().into())
        .ok_or("Couldn't compute home dir!".into())
}

pub fn load_json<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    #[cfg(test)]
    return Ok(T::default()); // don't load from file in tests

    if path.exists() {
        let s = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&s)?)
    } else {
        Ok(T::default())
    }
}

pub fn save_json(path: &Path, value: impl Serialize) -> Result<()> {
    #[cfg(test)]
    return Ok(()); // don't write to file in tests

    if !path.exists() {
        let parent = path
            .parent()
            .ok_or(format!("File has no parent dir: {path:?}"))?;
        fs::create_dir_all(parent)?; // create it if it doesn't exist
    }
    let contents = serde_json::to_string_pretty(&value)?;
    fs::write(path, contents)?;
    Ok(())
}
