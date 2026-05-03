mod implementations;
mod traits;
mod utils;

pub use implementations::json;
pub use implementations::simple;

pub use json::JsonUrlStore;
pub use simple::SimpleUrlStore;
pub use traits::AnnotatedUrl;
pub use traits::UrlStore;
