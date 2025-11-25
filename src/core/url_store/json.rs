use derive_more::derive::Display;

use crate::{
    Result,
    url_store::{
        UrlStore,
        traits::{AnnotatedUrl, UrlError},
        utils::{add_to_list, filter, remove_from_list},
    },
    utils::{load_json, save_json},
};
use std::{fmt::Display, path::PathBuf};

pub struct JsonUrlStore {
    /// Path to the file containing the urls.
    /// We need to keep a reference to this so we can load/save the URLs.
    path: PathBuf,
    /// URLs as an in-memory list
    urls: Vec<AnnotatedUrl>,
}
impl JsonUrlStore {
    /// RAII -- instantiating the struct also loads the urls from file.
    fn new(path: PathBuf) -> Result<Self> {
        let urls = load_json(&path)?;
        Ok(Self { path, urls })
    }
    /// Save the in-memory list of URLs to file.
    fn save(&self) -> Result<()> {
        save_json(&self.path, &self.urls)
    }
}
impl UrlStore for JsonUrlStore {
    fn add(&mut self, entry: AnnotatedUrl) -> Result<()> {
        add_to_list(&mut self.urls, entry.clone())?;
        self.save()
            .inspect(|_| log::info!("Added URL {entry}"))
    }
    fn remove(&mut self, query: &str) -> Result<Vec<AnnotatedUrl>> {
        let removed = remove_from_list(&mut self.urls, query)?;
        self.save()?;
        log::info!("removed URLs matching pattern {query}: {removed:#?}");
        return Ok(removed);
    }
    fn filter(&self, query: Option<&str>) -> Vec<AnnotatedUrl> {
        filter(&self.urls, query)
    }
}
