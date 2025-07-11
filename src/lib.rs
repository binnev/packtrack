#![allow(dead_code)]
#![allow(unused)]
#![feature(let_chains)]

mod core;
pub mod error;

pub use crate::error::{Error, Result};
pub use core::*;
