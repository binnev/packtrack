use derive_more::Display;

#[derive(Debug, Display)]
pub enum UrlError {
    #[display("'{_0}' is already in the URL store")]
    AlreadyInStore(String),

    #[display("'{_0}' was not found in the URL store")]
    NotFound(String),
}
