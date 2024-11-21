/// URL management
use crate::error::Result;
use crate::settings;
use std::path::Path;
use std::{env, fs};

pub async fn add(url: String) -> Result<()> {
    log::info!("adding {url}");
    let mut urls = load()?;
    urls.push(url);
    save(urls)?;
    Ok(())
}

pub async fn remove(url: String) -> Result<()> {
    log::info!("removing {url}");
    let mut urls = load()?;
    while let Some(idx) = urls.iter().position(|x| *x == url) {
        urls.remove(idx);
    }
    save(urls)?;
    Ok(())
}

pub async fn list() -> Result<()> {
    let urls = load()?;
    for url in urls {
        println!("{url}");
    }
    Ok(())
}

/// Load all URLs from the URLs file.
pub fn load() -> Result<Vec<String>> {
    let urls_file = settings::load()?.urls_file;
    let urls = fs::read_to_string(urls_file)?
        .lines()
        .map(|s| s.to_owned())
        .collect();
    Ok(urls)
}

pub fn save(urls: Vec<String>) -> Result<()> {
    let file = settings::load()?.urls_file;
    fs::write(file, urls.join("\n"))?;
    Ok(())
}
