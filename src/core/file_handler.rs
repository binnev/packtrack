use std::{fs, path::PathBuf};

use crate::Result;

/// Small trait that can be mocked in tests to prevent tests from doing IO.
pub trait FileHandler {
    /// Read the contents of the file
    fn load(&self, path: &PathBuf) -> Result<String>;
    /// Save to disk
    fn save(&self, path: &PathBuf, text: String) -> Result<()>;
}

pub struct TextFileHandler;
impl FileHandler for TextFileHandler {
    fn load(&self, path: &PathBuf) -> Result<String> {
        Ok(fs::read_to_string(path)?)
    }
    fn save(&self, path: &PathBuf, text: String) -> Result<()> {
        Ok(fs::write(path, text)?)
    }
}

// TODO: make it track number of calls, args, etc.
pub struct MockFileHandler;
impl FileHandler for MockFileHandler {
    fn load(&self, _path: &PathBuf) -> Result<String> {
        Ok("".into())
    }
    fn save(&self, _path: &PathBuf, _text: String) -> Result<()> {
        Ok(())
    }
}
