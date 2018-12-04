//! Implements the `sections` target which writes out parts of the syntax tree.
//!
//! This target operates on the same syntax tree as the `deps` target. It extracts
//! parts of the document tree marked by `<section />` tags and writes them to a
//! directory specified through the transformation settings in the YAML format.

use preamble::*;

use serde_json;
use std::io;

use structopt::StructOpt;

mod filter;
mod finder;

#[derive(Debug, StructOpt)]
pub struct SectionsArgs {
    /// Name of the section to extract.
    section: String,
}

/// Write document section to the filesystem.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct SectionsTarget {}

impl<'a> Target<&'a SectionsArgs, ()> for SectionsTarget {
    fn target_type(&self) -> TargetType {
        TargetType::Sections
    }

    fn export(
        &self,
        root: &Element,
        _: (),
        args: &'a SectionsArgs,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let inter = match filter::SectionFilter::extract(&args.section, root) {
            Some(inter) => inter,
            None => panic!(
                "could not find section \"{}\" in this document!",
                &args.section
            ),
        };
        serde_json::to_writer(out, &inter).expect("could not serialize section!");
        Ok(())
    }
}
