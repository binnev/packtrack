use packtrack::{
    error::Result,
    url_store::{AnnotatedUrl, JsonUrlStore, SimpleUrlStore, UrlStore},
};
use std::path::PathBuf;

/// Load URLs from file
pub fn load(file: &PathBuf) -> Result<Box<dyn UrlStore>> {
    if !file.exists() {
        return Err(format!("File {} does not exist!", file.display()).into());
    }
    let ext = match file.extension() {
        Some(s) => s
            .to_str()
            .ok_or(format!("Filename {file:?} is invalid unicode?!"))?,
        None => return Err(format!("File {file:?} has no extension!").into()),
    };
    let url_store: Box<dyn UrlStore> = match ext {
        "json" => Box::new(JsonUrlStore::new(file.clone())?),
        _ => Box::new(SimpleUrlStore::new(file.clone())?),
    };
    Ok(url_store)
}

/// Add a URL to the URLs file
pub fn add(file: &PathBuf, url: AnnotatedUrl) -> Result<()> {
    log::info!("adding {url}");
    let mut urls = load(file)?;
    urls.add(url)?;
    urls.save()
}

/// Remove a URL from file
pub fn remove(file: &PathBuf, pattern: String) -> Result<Vec<AnnotatedUrl>> {
    log::info!("removing URLs matching pattern {pattern}");
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
