//! Implements the `stats` target which extracts various statistical
//! information from the document tree.
use preamble::*;
use std::collections::{HashMap, HashSet};

use serde_json;
use std::io;

/// Dump stats to stdout as json.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct StatsTarget {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
struct Stats<'e> {
    #[serde(skip)]
    pub path: Vec<&'e Element>,

    /// The original document length
    pub line_count: usize,

    /// Number of files included
    pub image_count: usize,

    /// Number of templates used of a kind
    pub template_count: HashMap<String, usize>,

    /// List of internal reference targets
    pub reference_targets: HashSet<String>,

    /// List of reference targets with no corresponding anchors in the export
    pub unresolved_references: HashSet<String>,
}

impl<'e, 's: 'e> Traversion<'e, &'s Settings> for Stats<'e> {
    path_methods!('e);

    fn work(
        &mut self,
        root: &Element,
        settings: &'s Settings,
        _out: &mut io::Write,
    ) -> io::Result<bool> {
        match root {
            Element::InternalReference(ref iref) => {
                if is_file(iref, settings) {
                    self.image_count += 1
                } else {
                    let target = extract_plain_text(&iref.target);
                    let target = target.trim().trim_left_matches(":").to_string();

                    self.reference_targets.insert(target.clone());
                    let anchor = matching_anchor(&target, &settings.runtime.available_anchors);
                    if !anchor.is_some() {
                        let enc_target = mw_enc(&target);
                        // if a prefix exists, the target should exist as well,
                        // otherwise this reference is unresolved
                        let article_exists = settings
                            .runtime
                            .available_anchors
                            .iter()
                            .any(|anchor| enc_target.starts_with(anchor));
                        if article_exists {
                            self.unresolved_references.insert(target);
                        }
                    }
                }
            }
            Element::Template(ref template) => {
                let name = extract_plain_text(&template.name).trim().to_lowercase();
                let current = *self.template_count.get(&name).unwrap_or(&0);
                self.template_count.insert(name.clone(), current + 1);
            }
            _ => (),
        };
        Ok(true)
    }
}

impl Target for StatsTarget {
    fn include_sections(&self) -> bool {
        true
    }
    fn target_extension(&self) -> &str {
        "yml"
    }
    fn extension_for(&self, _ext: &str) -> &str {
        "dummy"
    }
    fn export<'a>(
        &self,
        root: &'a Element,
        settings: &Settings,
        _args: &[String],
        out: &mut io::Write,
    ) -> io::Result<()> {
        let mut stats = Stats::default();

        stats.line_count = root.get_position().end.line - 1;
        stats.run(root, settings, out)?;

        writeln!(
            out,
            "{}",
            serde_json::to_string(&stats).expect("could not serialize the stats struct")
        )
    }
}
