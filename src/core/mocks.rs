use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use crate::Result;
use serde_json::Value;

pub fn load_json(filename: &str) -> Result<Value> {
    let filename = filename.to_owned() + ".json";
    let s = load_text(&filename)?;
    let value: Value = serde_json::from_str(&s)?;
    Ok(value)
}

pub fn load_text(filename: &str) -> Result<String> {
    let path = Path::new("mocks").join(filename);
    let text = fs::read_to_string(&path).expect(&format!(
        "Couldn't load mock {path:?}. Working directory is {:?}",
        std::env::current_dir().unwrap()
    ));
    Ok(text)
}
