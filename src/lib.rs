extern crate mediawiki_parser;
extern crate mwparser_utils;
extern crate mfnf_sitemap;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate serde_json;

use mediawiki_parser::transformations::TResult;
use mediawiki_parser::Element;

mod target;
#[macro_use]
mod util;
#[macro_use]
mod settings;
mod latex;
mod deps;
mod sections;
mod pdf;
mod transformations;

#[cfg(test)]
mod test;

// common includes for submodules
mod preamble {
    pub use mediawiki_parser::Traversion;
    pub use target::Target;
    pub use settings::Settings;
    pub use mediawiki_parser::*;
    pub use std::io;
    pub use util::*;
}

// public exports
pub use target::Target;
pub use settings::{Settings, GeneralSettings, RuntimeSettings};


/// Available targets for mfnf-export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MFNFTargets {
    SectionDeps(deps::SectionDepsTarget),
    MediaDeps(deps::MediaDepsTarget),
    Latex(latex::LatexTarget),
    Sections(sections::SectionsTarget),
    PDF(pdf::PDFTarget),
}

impl MFNFTargets {
    /// Get the inner struct implementing the target trait.
    pub fn get_target(&self) -> &target::Target {
        match *self {
            MFNFTargets::SectionDeps(ref t) => t,
            MFNFTargets::MediaDeps(ref t) => t,
            MFNFTargets::Latex(ref t) => t,
            MFNFTargets::Sections(ref t) => t,
            MFNFTargets::PDF(ref t) => t,
        }
    }
}

/// Applies all transformations which should happen before section transclusion.
/// This is mostly tree normlization and is applied on all targets.
pub fn normalize(mut root: Element,
                 settings: &settings::Settings) -> TResult {

    root = transformations::normalize_template_names(root, settings)?;
    root = transformations::remove_file_prefix(root, settings)?;
    root = mwparser_utils::transformations::convert_template_list(root)?;
    if let Some(ref checker) = settings.runtime.tex_checker {
        root = mwparser_utils::transformations::normalize_math_formulas(root, checker)?;
    }
    Ok(root)
}

/// Applies transformations necessary for article output (e.g section transclusion).
pub fn compose(mut root: Element,
               settings: &settings::Settings) -> TResult {

    root = transformations::include_sections(root, settings)?;
    root = transformations::normalize_heading_depths(root, settings)?;
    root = transformations::remove_exclusions(root, settings)?;
    Ok(root)
}
