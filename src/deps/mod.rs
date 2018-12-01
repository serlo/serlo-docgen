//! Implementation of the `deps` target.
//!
//! The `deps` target is used to export a list of article dependencies.
//! It is applied to a syntax tree with only part of the export transformations applied.
//! Transformations such as section inclusion or heading depth normalization are excluded,
//! while others (e.g. tepmlate name translation, image prefix removal) are applied before
//! this target is executed.

use preamble::*;
use std::fs;
use std::path::PathBuf;

mod printers;

use self::printers::*;
use structopt::StructOpt;
use transformations;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "section-deps",
    about = "generate a makefile declaring included sections as prerequisites of `base_file`."
)]
struct SectionDepArgs {
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
pub struct SectionDepsTarget {}

impl Target for SectionDepsTarget {
    fn extension_for(&self, _ext: &str) -> &str {
        "%"
    }

    /// Extract dependencies from a raw source AST. Sections are
    /// not included at this point.
    fn export<'a>(
        &self,
        root: &'a Element,
        settings: &Settings,
        args: &[String],
        out: &mut io::Write,
    ) -> io::Result<()> {
        let args = SectionDepArgs::from_iter(args);

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

#[derive(Debug, StructOpt)]
#[structopt(
    name = "media-deps",
    about = "generate a makefile declaring included media as prerequisites of `base_file`."
)]
struct MediaDepArgs {
    /// The target file to generate prerequisites for.
    #[structopt(short = "b", long = "base-file")]
    base_file: String,

    /// The target to generate dependencies for.
    /// This determines media file extensions.
    target: String,
}

/// Writes a list of included media files in `make` format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MediaDepsTarget {}

impl Target for MediaDepsTarget {
    fn extension_for(&self, _ext: &str) -> &str {
        "%"
    }

    /// Extract dependencies from a raw source AST. Sections are
    /// not included at this point.
    fn export<'a>(
        &self,
        root: &'a Element,
        settings: &Settings,
        args: &[String],
        out: &mut io::Write,
    ) -> io::Result<()> {
        let args = MediaDepArgs::from_iter(args);
        let target = match settings.general.targets.get(&args.target) {
            Some(t) => t.get_target(),
            None => panic!("no target \"{}\" found / configured!", &args.target),
        };

        writeln!(out, "# dependencies for {}", &args.target)?;
        write!(out, "{}: ", &args.base_file)?;
        let mut printer = FilesPrinter::new(target);
        printer.run(&root, settings, out)?;
        writeln!(out)
    }
}
