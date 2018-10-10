//! The `anchors` target.
//!
//! The `anchors` target extracts all valid anchors (places which can be linked to)
//! from an article. This allows to detect wether the target of a internal reference
//! is available in the export or not.

use preamble::*;
use std::process;

use transformations;

/// Writes a list of valid anchors to the output.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AnchorsTarget {}

impl Target for AnchorsTarget {
    fn target_extension(&self) -> &str {
        "anchors"
    }
    fn include_sections(&self) -> bool {
        true
    }
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
        // check of supplied targets, throw an error if target is not found.
        let mut target_list = args.to_vec();

        for (target_name, _target) in &settings.general.targets {
            if !args.contains(&target_name) {
                continue;
            }
            target_list = target_list
                .iter()
                .filter(|s| s != &target_name)
                .map(|s| s.clone())
                .collect();
            // apply exclusions
            let root = {
                let mut new_settings = Settings::default();
                new_settings.runtime.markers = settings.runtime.markers.clone();
                new_settings.runtime.target_name = target_name.to_string();
                transformations::remove_exclusions(root.clone(), &new_settings)
                    .expect("error applying exclusions!")
            };

            let mut printer = AnchorPrinter::default();
            printer.run(&root, settings, out)?;

            writeln!(out)?;
        }

        if !target_list.is_empty() {
            eprintln!(
                "The following targets are not defined: {}",
                &target_list.join(", ")
            );
            process::exit(2);
        }
        Ok(())
    }
}

/// prints all possible link targets (anchors) within this article.
#[derive(Default)]
pub struct AnchorPrinter<'b> {
    pub path: Vec<&'b Element>,
}

impl<'a, 'b: 'a> Traversion<'a, &'b Settings> for AnchorPrinter<'a> {
    path_methods!('a);

    fn work(
        &mut self,
        root: &Element,
        settings: &'b Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        if let Some(anchor) = extract_anchor(root, settings) {
            writeln!(out, "{}", anchor)?;
        }
        Ok(true)
    }
}
