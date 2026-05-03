mod error;
mod implementations;
mod models;
mod traits;
mod utils;

pub use implementations::json;
pub use implementations::simple;

pub use error::UrlError;
pub use json::JsonUrlStore;
pub use models::AnnotatedUrl;
pub use simple::SimpleUrlStore;
pub use traits::UrlStore;
