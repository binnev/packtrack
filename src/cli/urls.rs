use packtrack::{
    error::Result,
    url_store::{AnnotatedUrl, FileUrlStore, UrlStore},
};
use std::path::PathBuf;

// TODO: This should be in core
/// Load URLs from file
pub fn load(file: &PathBuf) -> Result<Box<dyn UrlStore>> {
    let store = FileUrlStore::new(file.clone())?;
    Ok(Box::new(store))
}

/// Add a URL to the URLs file
pub fn add(file: &PathBuf, url: AnnotatedUrl) -> Result<()> {
    let mut urls = load(file)?;
    urls.add(url)?;
    urls.save()
}

/// Remove a URL from file
pub fn remove(file: &PathBuf, pattern: String) -> Result<Vec<AnnotatedUrl>> {
    let mut urls = load(file)?;
    let removed = urls.remove(&pattern)?;
    urls.save()?;
    Ok(removed)
}

/// Filter URLs from file
pub fn filter(
    file: &PathBuf,
    query: Option<&str>,
) -> Result<Vec<AnnotatedUrl>> {
    let urls = load(file)?;
    Ok(urls.filter(query))
}
