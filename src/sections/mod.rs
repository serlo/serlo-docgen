//! Implements the `sections` target which writes out parts of the syntax tree.
//!
//! This target operates on the same syntax tree as the `deps` target. It extracts
//! parts of the document tree marked by `<section />` tags and writes them to a
//! directory specified through the transformation settings in the YAML format.

use preamble::*;

use serde_json;
use std::fs::DirBuilder;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;

use structopt::StructOpt;

mod filter;
mod finder;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "sections",
    about = "extract a section from a document."
)]
struct Args {
    /// Title of the document.
    title: String,
    /// Name of the section to extract.
    section: String,
    /// Revision of the document.
    revision: String,
}

/// Write document section to the filesystem.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct SectionsTarget {}

impl Target for SectionsTarget {
    fn extension_for(&self, _ext: &str) -> &str {
        "%"
    }
    fn export<'a>(
        &self,
        root: &'a Element,
        _settings: &Settings,
        args: &[String],
        out: &mut io::Write,
    ) -> io::Result<()> {
        let args = Args::from_iter(args);

        let inter = match filter::SectionFilter::extract(&args.section, root) {
            Some(inter) => inter,
            None => panic!(
                "could not find section \"{}\" in this document!",
                &args.section
            ),
        };

        let path = PathBuf::new()
            .join(filename_to_make(&args.title))
            .join(filename_to_make(&args.section))
            .join(filename_to_make(&args.revision))
            .with_extension("json");

        DirBuilder::new().recursive(true).create(&path)?;

        Ok(serde_json::to_writer(out, &inter).expect("could not serialize section!"))
    }
}
