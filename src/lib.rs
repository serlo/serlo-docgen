extern crate mediawiki_parser;
extern crate mfnf_sitemap;
extern crate mfnf_template_spec;
extern crate mwparser_utils;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate serde_json;
#[cfg(test)]
extern crate serde_yaml;

mod meta;
mod target;
#[macro_use]
mod util;
#[macro_use]
mod settings;
mod anchors;
mod compose;
mod deps;
mod html;
mod latex;
mod normalize;
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
    Anchors(anchors::AnchorsTarget),
    Normalize(normalize::NormalizeTarget),
    Compose(compose::ComposeTarget),
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
            MFNFTargets::Anchors(ref t) => t,
            MFNFTargets::Normalize(ref t) => t,
            MFNFTargets::Compose(ref t) => t,
        }
    }
}
