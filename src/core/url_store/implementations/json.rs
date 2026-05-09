use crate::{
    Result,
    file_handler::{FileHandler, TextFileHandler},
    url_store::{
        UrlStore,
        models::AnnotatedUrl,
        utils::{add_to_list, filter, remove_from_list},
    },
};
use std::path::PathBuf;

pub struct JsonUrlStore {
    /// Path to the file containing the urls.
    /// We need to keep a reference to this so we can load/save the URLs.
    path:         PathBuf,
    /// URLs as an in-memory list
    urls:         Vec<AnnotatedUrl>,
    file_handler: Box<dyn FileHandler>,
}
impl JsonUrlStore {
    /// RAII -- instantiating the struct also loads the urls from file.
    pub fn new(path: PathBuf) -> Result<Self> {
        #[allow(unused_mut)]
        let mut file_handler: Box<dyn FileHandler> = Box::new(TextFileHandler);

        #[cfg(test)]
        {
            use crate::file_handler::MockFileHandler;
            file_handler = Box::new(MockFileHandler);
        }

        let text = file_handler.load(&path)?;
        let urls = serde_json::from_str(&text)?;
        Ok(Self {
            path,
            urls,
            file_handler,
        })
    }
}
impl UrlStore for JsonUrlStore {
    fn add(&mut self, entry: AnnotatedUrl) -> Result<()> {
        add_to_list(&mut self.urls, entry.clone())?;
        log::info!("Added URL {entry}");
        Ok(())
    }
    fn remove(&mut self, query: &str) -> Result<Vec<AnnotatedUrl>> {
        let removed = remove_from_list(&mut self.urls, query)?;
        log::info!(
            "Removed {} URLs matching pattern {query}: {removed:#?}",
            removed.len()
        );
        return Ok(removed);
    }
    fn filter(&self, query: Option<&str>) -> Vec<AnnotatedUrl> {
        filter(&self.urls, query)
    }
    /// Save the in-memory list of URLs to file.
    fn save(&self) -> Result<()> {
        let text = serde_json::to_string(&self.urls)?;
        self.file_handler.save(&self.path, text)
    }
}
