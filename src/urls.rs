/// URL management
use crate::error::Result;
use crate::settings;
use std::ops::Index;
use std::path::Path;
use std::{env, fs};

pub async fn add(url: String) -> Result<()> {
    log::info!("adding {url}");
    let mut urls = load()?;
    urls.push(url);
    save(urls)?;
    Ok(())
}

pub async fn remove(pattern: String) -> Result<Vec<String>> {
    log::info!("removing URLs matching pattern {pattern}");
    let mut urls = load()?;
    let removed = _remove(&mut urls, &pattern);
    log::info!("removed URLs: {removed:?}");
    save(urls)?;
    Ok(removed)
}
// making this a separate function so it's easier to test
fn _remove(urls: &mut Vec<String>, pattern: &str) -> Vec<String> {
    let mut removed: Vec<String> = vec![];
    while let Some(idx) = urls
        .iter()
        .position(|x| x.contains(&pattern))
    {
        let url = urls.remove(idx);
        log::debug!("Removed URL: {url}");
        removed.push(url);
    }
    removed
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_pattern() {
        let mut urls: Vec<String> = vec![
            "www.example.com".into(),
            "www.ups.org".into(),
            "www.dhl.org".into(),
        ];
        let removed = _remove(&mut urls, ".org");
        assert_eq!(removed, vec!["www.ups.org", "www.dhl.org",]);
        let expected = vec!["www.example.com"];
        assert_eq!(urls, expected);
    }
    #[test]
    fn test_remove_exact() {
        let mut urls: Vec<String> = vec![
            "www.example.com".into(),
            "www.ups.org".into(),
            "www.dhl.org".into(),
        ];
        let removed = _remove(&mut urls, "www.dhl.org");
        assert_eq!(removed, vec!["www.dhl.org",]);
        let expected = vec!["www.example.com", "www.ups.org"];
        assert_eq!(urls, expected);
    }
}
