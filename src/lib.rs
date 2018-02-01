extern crate mediawiki_parser;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use mediawiki_parser::transformations::TResult;
use mediawiki_parser::Element;

mod target;
#[macro_use]
mod settings;
#[macro_use]
mod util;
mod latex;
mod deps;
mod sections;
mod transformations;

#[cfg(test)]
mod test;

// common includes for submodules
mod preamble {
    pub use mediawiki_parser::Traversion;
    pub use target::Target;
    pub use settings::Settings;
    pub use mediawiki_parser::Element;
    pub use std::io;
    pub use util::*;
}

// public exports
pub use target::Target;
pub use settings::Settings;


/// Available targets for mfnf-export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MFNFTargets {
    Dependencies(deps::DepsTarget),
    Latex(latex::LatexTarget),
    Sections(sections::SectionsTarget),
}

impl MFNFTargets {
    /// Get the inner struct implementing the target trait.
    pub fn get_target(&self) -> &target::Target {
        match *self {
            MFNFTargets::Dependencies(ref t) => t,
            MFNFTargets::Latex(ref t) => t,
            MFNFTargets::Sections(ref t) => t,
        }
    }
}

/// Applies all transformations which should happen before section transclusion.
/// This is mostly tree normlization and is applied on all targets.
pub fn normalize(mut root: Element,
                 settings: &settings::Settings) -> TResult {

    root = transformations::normalize_template_names(root, settings)?;
    root = transformations::translate_templates(root, settings)?;
    root = transformations::normalize_template_title(root, settings)?;
    root = transformations::remove_file_prefix(root, settings)?;
    Ok(root)
}

/// Applies transformations necessary for article output (e.g section transclusion).
pub fn compose(mut root: Element,
               settings: &settings::Settings) -> TResult {

    root = transformations::include_sections(root, settings)?;
    root = transformations::normalize_heading_depths(root, settings)?;
    Ok(root)
}
