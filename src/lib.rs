extern crate mediawiki_parser;
extern crate config;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use mediawiki_parser::ast::Element;
use mediawiki_parser::transformations::TResult;

/// Structures for configuration of transformations.
#[macro_use]
pub mod settings;

pub mod latex;
pub mod deps;
pub mod sections;
mod util;
mod transformations;

/// Applies all transformations which should happen before section transclusion.
/// This is mostly tree normlization and is applied on all targets.
pub fn apply_universal_transformations(mut root: Element,
                                       settings: &settings::Settings) -> TResult {
    root = transformations::normalize_template_names(root, settings)?;
    root = transformations::translate_templates(root, settings)?;
    root = transformations::normalize_template_title(root, settings)?;
    root = transformations::remove_file_prefix(root, settings)?;
    Ok(root)
}

/// Applies transformations necessary for article output (e.g section transclusion).
pub fn apply_output_transformations(mut root: Element,
                                    settings: &settings::Settings) -> TResult {
    root = transformations::include_sections(root, settings)?;
    root = transformations::normalize_heading_depths(root, settings)?;
    Ok(root)
}
