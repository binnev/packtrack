use derive_more::Display;
use packtrack::{
    error::{Error, Result},
    url_store::{AnnotatedUrl, JsonUrlStore, SimpleUrlStore, UrlStore},
};
use std::path::PathBuf;

/// Load URLs from file
pub fn load(file: &PathBuf) -> Result<Box<dyn UrlStore>> {
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
    urls.add(url)
}

/// Remove a URL from file
pub fn remove(file: &PathBuf, pattern: String) -> Result<Vec<AnnotatedUrl>> {
    log::info!("removing URLs matching pattern {pattern}");
    let mut urls = load(file)?;
    let removed = urls.remove(&pattern)?;
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

#[derive(Debug, Display)]
pub enum UrlError {
    #[display("'{_0}' is already in the URLs file")]
    AlreadyInFile(String),

    #[display("'{_0}' was not found in the URLs file")]
    NotFound(String),
}
impl From<UrlError> for Error {
    fn from(e: UrlError) -> Error {
        Error::Custom(e.to_string())
    }
}
