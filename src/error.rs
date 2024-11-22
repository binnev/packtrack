use std::sync::PoisonError;

use derive_more::From;

use crate::urls::UrlError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    // -- Internals

    // URL management
    #[from]
    Url(UrlError),

    #[from]
    Custom(String),

    // -- Externals
    #[from]
    Chrono(chrono::ParseError),

    #[from]
    SerdeJson(serde_json::Error),

    #[from]
    Reqwest(reqwest::Error),

    #[from]
    Regex(regex::Error),

    #[from]
    Io(std::io::Error),
}

impl PartialEq for Error {
    // Just do string eq for now
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Error::Custom(a), Error::Custom(b)) => a == b,
            (a, b) => a.to_string() == b.to_string(),
        }
    }
}

impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self::Custom(val.to_string())
    }
}

impl core::fmt::Display for Error {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter,
    ) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
