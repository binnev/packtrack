use crate::{
    Result,
    file_handler::{FileHandler, TextFileHandler},
    url_store::{
        UrlStore,
        file_url_store::url_serializer::{
            JsonUrlSerializer, SimpleUrlSerializer, UrlSerializer,
        },
        models::AnnotatedUrl,
        utils::{add_to_list, filter, remove_from_list},
    },
};
use std::path::PathBuf;

/// URL store which stores its urls as a `Vec<AnnotatedUrl>` in memory, and as a
/// text file on disk. Handles different file formats.
pub struct FileUrlStore {
    path:         PathBuf,
    urls:         Vec<AnnotatedUrl>,
    file_handler: Box<dyn FileHandler>,
    serializer:   Box<dyn UrlSerializer>,
}

impl FileUrlStore {
    /// RAII -- instantiating the struct also loads the urls from file.
    pub fn new(path: PathBuf) -> Result<Self> {
        #[allow(unused_mut)]
        let mut file_handler: Box<dyn FileHandler> = Box::new(TextFileHandler);

        // Use a mock file handler in tests to prevent tests doing IO
        #[cfg(test)]
        {
            use crate::file_handler::MockFileHandler;
            file_handler = Box::new(MockFileHandler);
        }

        let serializer = Self::select_serializer(&path)?;
        let urls = serializer.deserialize(&file_handler.load(&path)?)?;
        Ok(Self {
            path,
            urls,
            file_handler,
            serializer,
        })
    }

    /// Select the appropriate serializer based on the file's extension
    fn select_serializer(file: &PathBuf) -> Result<Box<dyn UrlSerializer>> {
        if !file.exists() {
            return Err(
                format!("File {} does not exist!", file.display()).into()
            );
        }
        let ext = match file.extension() {
            Some(s) => s
                .to_str()
                .ok_or(format!("Filename {file:?} is invalid unicode?!"))?,
            None => {
                return Err(format!("File {file:?} has no extension!").into());
            }
        };
        let serializer: Box<dyn UrlSerializer> = match ext {
            "json" => Box::new(JsonUrlSerializer),
            _ => Box::new(SimpleUrlSerializer),
        };
        Ok(serializer)
    }
}

impl UrlStore for FileUrlStore {
    fn add(&mut self, entry: AnnotatedUrl) -> Result<()> {
        let path = &self.path.display();
        log::debug!("Adding URL {entry} to {path}");
        add_to_list(&mut self.urls, entry.clone())
            .inspect(|_| log::info!("Added URL {entry} to {path}"))
            .inspect_err(|err| {
                log::warn!("Error adding URL {entry} to {path}: {err}")
            })
    }
    fn remove(&mut self, query: &str) -> Result<Vec<AnnotatedUrl>> {
        let path = &self.path.display();
        log::debug!("Removing URLs from {path} matching pattern {query}");
        remove_from_list(&mut self.urls, query)
            .inspect(|removed| log::info!(
                "Removed {} URLs from {path} matching pattern {query}: {removed:#?}", 
                removed.len(),
            ))
            .inspect_err(|err| log::warn!(
                "Error removing URLs from {path} matching pattern {query}: {err}")
            )
    }
    fn filter(&self, query: Option<&str>) -> Vec<AnnotatedUrl> {
        filter(&self.urls, query)
    }
    fn save(&self) -> Result<()> {
        let path = &self.path.display();
        self.serializer
            .serialize(&self.urls)
            .and_then(|text| self.file_handler.save(&self.path, text))
            .inspect(|_| log::info!("Saved URLs to {path}"))
            .inspect_err(|err| {
                log::error!("Error saving URLs to {path}: {err}")
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        file_handler::MockFileHandler,
        url_store::file_url_store::url_serializer::SimpleUrlSerializer,
    };

    use super::*;

    impl FileUrlStore {
        fn for_test(serializer: Box<dyn UrlSerializer>) -> Self {
            Self {
                path: "/dev/null".into(),
                urls: vec![],
                file_handler: Box::new(MockFileHandler),
                serializer,
            }
        }
        fn for_test_simple() -> Self {
            let serializer = Box::new(SimpleUrlSerializer);
            Self::for_test(serializer)
        }
    }

    #[test]
    fn test_simple_url_store() -> Result<()> {
        let mut s = FileUrlStore::for_test_simple();
        let url = AnnotatedUrl {
            url:         "example.com".into(),
            description: None,
            created:     None,
        };
        s.add(url.clone())
            .expect("The first add should work");
        assert_eq!(s.urls.len(), 1);
        let result = s.add(url.clone());
        assert!(result.is_err());
        Ok(())
    }
}
