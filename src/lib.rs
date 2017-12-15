extern crate mediawiki_parser;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use mediawiki_parser::ast::Element;
use mediawiki_parser::transformations::TResult;

/// Structures for configuration of transformations.
pub mod settings;

mod transformations;

/// Applies all MFNF-Specific transformations.
pub fn apply_transformations(root: Element, settings: &settings::Settings) -> TResult {
    transformations::normalize_template_names(root, settings)
}
