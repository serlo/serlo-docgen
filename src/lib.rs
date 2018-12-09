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
    pub use crate::settings::Settings;
    pub use crate::util::*;
    pub use crate::Target;
    pub use crate::TargetType;
    pub use mediawiki_parser::Traversion;
    pub use mediawiki_parser::*;
    pub use serde_derive::{Deserialize, Serialize};
    pub use std::io;
}

use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::io;

// public exports
pub use crate::anchors::{AnchorsArgs, AnchorsTarget};
pub use crate::compose::{ComposeArgs, ComposeTarget};
pub use crate::deps::{MediaDepArgs, MediaDepTarget, SectionDepArgs, SectionDepTarget};
pub use crate::html::{HTMLArgs, HTMLTarget};
pub use crate::latex::{LatexArgs, LatexTarget};
pub use crate::normalize::{NormalizeArgs, NormalizeTarget};
pub use crate::pdf::{PDFArgs, PDFTarget};
pub use crate::sections::{SectionsArgs, SectionsTarget};
pub use crate::settings::Settings;
pub use crate::stats::{StatsArgs, StatsTarget};

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
    #[serde(rename = "pdf")]
    PDF,
    Stats,
    #[serde(rename = "html")]
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
