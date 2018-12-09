//! Implements the `compose` target.
//!
//! This target is more a transformation than an export target. The output
//! is the article with section inclusions and heading exclusions applied.

mod transformations;

use crate::preamble::*;
use crate::transformations::remove_exclusions;
use mediawiki_parser::transformations::TResult;
use mfnf_sitemap::Markers;
use std::fs;
use std::path::PathBuf;
use std::process;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ComposeArgs {
    /// Path to article markers (includes / excludes).
    #[structopt(parse(from_os_str), short = "m", long = "markers")]
    marker_path: PathBuf,
    /// Path to the article sections directory.
    #[structopt(parse(from_os_str), short = "s", long = "section-path")]
    section_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ComposeTarget {}

/// Prepare the article for rendering.
pub fn compose(mut root: Element, section_path: &PathBuf, markers: &Markers) -> TResult {
    root = transformations::include_sections(root, section_path)?;
    root = transformations::normalize_heading_depths(root, ())?;
    root = remove_exclusions(root, &markers)?;
    Ok(root)
}

impl<'a> Target<&'a ComposeArgs, ()> for ComposeTarget {
    fn target_type(&self) -> TargetType {
        TargetType::Compose
    }

    fn export(
        &self,
        root: &Element,
        _: (),
        args: &'a ComposeArgs,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let markers = {
            let file = fs::File::open(&args.marker_path)?;
            serde_json::from_reader(&file).expect("Error reading markers:")
        };

        match compose(root.clone(), &args.section_path, &markers) {
            Ok(result) => serde_json::to_writer(out, &result).expect("could not serialize result!"),
            Err(err) => {
                eprintln!("{}", &err);
                serde_json::to_writer(out, &err).expect("could not serialize error!");
                process::exit(1);
            }
        };
        Ok(())
    }
}
