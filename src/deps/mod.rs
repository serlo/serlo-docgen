//! Implementation of the `deps` target.
//!
//! The `deps` target is used to export a list of article dependencies.
//! It is applied to a syntax tree with only part of the export transformations applied.
//! Transformations such as section inclusion or heading depth normalization are excluded,
//! while others (e.g. tepmlate name translation, image prefix removal) are applied before
//! this target is executed.

use crate::preamble::*;
use std::fs;
use std::path::PathBuf;

mod printers;

use self::printers::*;
use crate::transformations;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct SectionDepArgs {
    /// Path to article markers (includes / excludes).
    #[structopt(parse(from_os_str), short = "m", long = "markers")]
    marker_path: PathBuf,
    /// Path to the article sections directory.
    #[structopt(parse(from_os_str), short = "s", long = "section-path")]
    section_path: PathBuf,
    /// The target file to generate prerequisites for.
    #[structopt(short = "b", long = "base-file")]
    base_file: String,
}

/// Writes a list of included sections in `make` format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SectionDepTarget {}

impl<'a> Target<&'a SectionDepArgs, ()> for SectionDepTarget {
    fn target_type(&self) -> TargetType {
        TargetType::SectionDeps
    }
    /// Extract dependencies from a raw source AST. Sections are
    /// not included at this point.
    fn export(
        &self,
        root: &Element,
        _: (),
        args: &'a SectionDepArgs,
        out: &mut dyn io::Write,
    ) -> io::Result<()> {
        let markers = {
            let file = fs::File::open(&args.marker_path)?;
            serde_json::from_reader(&file).expect("Error reading markers:")
        };
        // apply exclusions
        let root = transformations::remove_exclusions(root.clone(), &markers)
            .expect("error applying exclusions!");

        write!(out, "{}: ", &args.base_file)?;
        let mut printer = InclusionPrinter::default();
        printer.run(&root, &args.section_path, out)?;
        writeln!(out)
    }
}

fn parse_target_type(input: &str) -> serde_json::Result<TargetType> {
    serde_json::from_str(&format!("\"{}\"", input))
}

#[derive(Debug, StructOpt)]
pub struct MediaDepArgs {
    /// The target file to generate prerequisites for.
    #[structopt(short = "b", long = "base-file")]
    base_file: String,

    /// The target to generate dependencies for.
    /// This determines media file extensions.
    #[structopt(parse(try_from_str = "parse_target_type"))]
    target_type: TargetType,
}

/// Writes a list of included media files in `make` format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MediaDepTarget {}

impl<'a, 's> Target<&'a MediaDepArgs, &'s Settings> for MediaDepTarget {
    fn target_type(&self) -> TargetType {
        TargetType::MediaDeps
    }
    /// Extract dependencies from a raw source AST. Sections are
    /// not included at this point.
    fn export(
        &self,
        root: &Element,
        settings: &'s Settings,
        args: &'a MediaDepArgs,
        out: &mut dyn io::Write,
    ) -> io::Result<()> {
        writeln!(out, "# dependencies for {}", &args.target_type)?;
        write!(out, "{}: ", &args.base_file)?;
        let mut printer = FilesPrinter::new(args.target_type);
        printer.run(&root, settings, out)?;
        writeln!(out)
    }
}
