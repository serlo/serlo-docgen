extern crate mediawiki_parser;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use mediawiki_parser::ast::Element;


/// Structures for configuration of transformations.
pub mod settings;

mod transformations;

/// Applies all MFNF-Specific transformations.
pub fn apply_transformations(root: Element, settings: &settings::Settings) -> Element {
    root
}
