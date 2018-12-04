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
extern crate structopt;

mod meta;
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
    pub use util::*;
    pub use Target;
    pub use TargetType;
}

// public exports
pub use settings::Settings;
use std::fmt;
use std::io;

pub use anchors::{AnchorsArgs, AnchorsTarget};
pub use compose::{ComposeArgs, ComposeTarget};
pub use deps::{MediaDepArgs, MediaDepTarget, SectionDepArgs, SectionDepTarget};
pub use html::{HTMLArgs, HTMLTarget};
pub use latex::{LatexArgs, LatexTarget};
pub use normalize::{NormalizeArgs, NormalizeTarget};
pub use pdf::{PDFArgs, PDFTarget};
pub use sections::{SectionsArgs, SectionsTarget};
pub use stats::{StatsArgs, StatsTarget};

/// Marks an exportable target type.
pub trait Target<A, S> {
    fn target_type(&self) -> TargetType;
    /// export the the ast to `out`.
    fn export(
        &self,
        root: &mediawiki_parser::Element,
        settings: S,
        args: A,
        out: &mut io::Write,
    ) -> io::Result<()>;
}

/// Available targets for mfnf-export.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetType {
    Sections,
    SectionDeps,
    MediaDeps,
    Normalize,
    Compose,
    Anchors,
    Latex,
    PDF,
    Stats,
    HTML,
}

/// Possible target configuration structs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Targets {
    Sections(SectionsTarget),
    SectionDeps(SectionDepTarget),
    MediaDeps(MediaDepTarget),
    Normalize(NormalizeTarget),
    Compose(ComposeTarget),
    Anchors(AnchorsTarget),
    Latex(LatexTarget),
    PDF(PDFTarget),
    Stats(StatsTarget),
    HTML(HTMLTarget),
}

impl std::str::FromStr for TargetType {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl fmt::Display for TargetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            &serde_json::to_string(self).expect("could not serialize TargetType!")
        )
    }
}
