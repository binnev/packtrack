use std::{num::ParseIntError, sync::PoisonError};

use derive_more::{From, derive::Display};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Display)]
pub enum Error {
    // -- Internals
    #[from]
    #[display("{_0}")]
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

    #[from]
    Parse(ParseIntError),
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

impl std::error::Error for Error {}
