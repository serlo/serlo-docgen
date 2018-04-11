//! This library provides common, Mathe-für-Nicht-Freaks specific code.

extern crate mediawiki_parser;
extern crate serde;
#[macro_use]
extern crate serde_derive;

#[macro_use]
pub mod util;
#[macro_use]
mod spec_utils;
mod spec;

pub use spec::*;

