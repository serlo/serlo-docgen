//! Implements the `compose` target.
//!
//! This target is more a transformation than an export target. The output
//! is the article with section inclusions and heading exclusions applied.

mod transformations;

use mediawiki_parser::transformations::TResult;
use preamble::*;
use std::process;
use transformations::remove_exclusions;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ComposeTarget {}

/// Prepare the article for rendering.
pub fn compose(mut root: Element, settings: &Settings) -> TResult {
    root = transformations::include_sections(root, settings)?;
    root = transformations::normalize_heading_depths(root, settings)?;
    root = remove_exclusions(root, settings)?;
    Ok(root)
}

impl Target for ComposeTarget {
    fn target_extension(&self) -> &str {
        "json"
    }
    fn include_sections(&self) -> bool {
        true
    }
    fn extension_for(&self, _ext: &str) -> &str {
        "%"
    }

    fn export<'a>(
        &self,
        root: &'a Element,
        settings: &Settings,
        _args: &[String],
        out: &mut io::Write,
    ) -> io::Result<()> {
        match compose(root.clone(), settings) {
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
