extern crate mediawiki_parser;
extern crate mfnf_sitemap;
extern crate mfnf_template_spec;
extern crate mwparser_utils;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;

use mediawiki_parser::transformations::TResult;
use mediawiki_parser::Element;

mod meta;
mod target;
#[macro_use]
mod util;
#[macro_use]
mod settings;
mod deps;
mod html;
mod latex;
mod pdf;
mod sections;
mod stats;
mod transformations;

#[cfg(test)]
mod test;

// common includes for submodules
mod preamble {
    pub use mediawiki_parser::Traversion;
    pub use mediawiki_parser::*;
    pub use settings::Settings;
    pub use std::io;
    pub use target::Target;
    pub use util::*;
}

// public exports
pub use settings::{GeneralSettings, RuntimeSettings, Settings};
pub use target::Target;

/// Available targets for mfnf-export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MFNFTargets {
    SectionDeps(deps::SectionDepsTarget),
    MediaDeps(deps::MediaDepsTarget),
    Latex(latex::LatexTarget),
    Sections(sections::SectionsTarget),
    PDF(pdf::PDFTarget),
    Stats(stats::StatsTarget),
    HTML(html::HTMLTarget),
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
            MFNFTargets::Stats(ref t) => t,
            MFNFTargets::HTML(ref t) => t,
        }
    }
}

/// Applies all transformations which should happen before section transclusion.
/// This is mostly tree normlization and is applied on all targets.
pub fn normalize(mut root: Element, settings: &settings::Settings) -> TResult {
    root = transformations::normalize_template_names(root, settings)?;
    root = mwparser_utils::transformations::convert_template_list(root)?;
    if let Some(ref checker) = settings.runtime.tex_checker {
        root = mwparser_utils::transformations::normalize_math_formulas(root, checker)?;
    }
    root = transformations::remove_whitespace_trailers(root, settings)?;
    Ok(root)
}

/// Applies transformations necessary for article output (e.g section transclusion).
pub fn compose(mut root: Element, settings: &settings::Settings) -> TResult {
    root = transformations::include_sections(root, settings)?;
    root = transformations::normalize_heading_depths(root, settings)?;
    root = transformations::remove_exclusions(root, settings)?;
    root = transformations::resolve_interwiki_links(root, settings)?;
    Ok(root)
}
