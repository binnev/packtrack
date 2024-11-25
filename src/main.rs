#![allow(dead_code)]
#![allow(unused)]
#![feature(let_chains)]

mod api;
mod cache;
mod cached_tracker;
mod cli;
mod error;
mod mocks;
mod settings;
mod tracker;
mod urls;
mod utils;

use crate::error::{Error, Result};

#[tokio::main]
async fn main() -> Result<()> {
    cli::main().await
}
