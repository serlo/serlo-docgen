//! The `anchors` target.
//!
//! The `anchors` target extracts all valid anchors (places which can be linked to)
//! from an article. This allows to detect wether the target of a internal reference
//! is available in the export or not.

use preamble::*;
use serde_yaml;
use std::process;

use mfnf_template_spec::{parse_template, KnownTemplate};
use transformations;

/// Writes a list of valid anchors to the output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AnchorsTarget {
    /// caption text used in a reference to an anchor.
    anchor_caption: String,
}

impl Default for AnchorsTarget {
    fn default() -> AnchorsTarget {
        AnchorsTarget {
            anchor_caption: "Anker".into(),
        }
    }
}

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
        // apply exclusions
        let root = {
            let result = transformations::remove_exclusions(root.clone(), &settings);
            match result {
                Err(err) => {
                    eprintln!("{}", &err);
                    println!(
                        "{}",
                        serde_yaml::to_string(&err).expect("Could not serialize error!")
                    );
                    process::exit(1);
                }
                Ok(tree) => tree,
            }
        };

        let mut printer = AnchorPrinter::new(self);
        printer.run(&root, settings, out)?;

        Ok(())
    }
}

/// prints all possible link targets (anchors) within this article.
pub struct AnchorPrinter<'b, 't> {
    pub path: Vec<&'b Element>,
    pub target: &'t AnchorsTarget,
}

impl<'a, 'b: 'a, 't> Traversion<'a, &'b Settings> for AnchorPrinter<'a, 't> {
    path_methods!('a);

    fn work(
        &mut self,
        root: &Element,
        settings: &'b Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        match root {
            Element::Document(_) => {
                writeln!(out, "{}", &settings.runtime.document_title)?;
            }
            Element::Heading(ref heading) => {
                let text = extract_plain_text(&heading.caption);
                writeln!(out, "{}#{}", &settings.runtime.document_title, &text)?;
            }
            Element::Template(ref template) => {
                if let Some(KnownTemplate::Anchor(ref anchor)) = parse_template(template) {
                    writeln!(
                        out,
                        "{}#{}:{}",
                        &settings.runtime.document_title,
                        &self.target.anchor_caption,
                        &extract_plain_text(&anchor.ref1),
                    )?;
                }
            }
            _ => (),
        }
        Ok(true)
    }
}

impl<'e, 't> AnchorPrinter<'e, 't> {
    pub fn new(target: &'t AnchorsTarget) -> AnchorPrinter<'e, 't> {
        AnchorPrinter {
            path: vec![],
            target,
        }
    }
}
