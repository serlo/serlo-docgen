//! Implements the `stats` target which extracts various statistical
//! information from the document tree.
use preamble::*;
use std::collections::{HashMap, HashSet};

use serde_json;
use std::io;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct StatsArgs {
    /// Title of the document beeing processed.
    document_title: String,

    /// Path to a list of link targets (anchors) available in the export.
    #[structopt(parse(try_from_str = "load_anchor_set"))]
    available_anchors: HashSet<String>,
}

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

impl<'e, 's: 'e, 'a> Traversion<'e, (&'s Settings, &'a StatsArgs)> for Stats<'e> {
    path_methods!('e);

    fn work(
        &mut self,
        root: &Element,
        params: (&'s Settings, &'a StatsArgs),
        _out: &mut io::Write,
    ) -> io::Result<bool> {
        let (settings, args) = params;
        match root {
            Element::InternalReference(ref iref) => {
                if is_file(iref, settings) {
                    self.image_count += 1
                } else {
                    let target = extract_plain_text(&iref.target);
                    let target = target.trim().trim_left_matches(':').to_string();

                    self.reference_targets.insert(target.clone());
                    let anchor = matching_anchor(&target, &args.available_anchors);
                    if anchor.is_none() {
                        let enc_target = mw_enc(&target);
                        // if a prefix exists, the target should exist as well,
                        // otherwise this reference is unresolved
                        let article_exists = args
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

impl<'a, 's> Target<&'a StatsArgs, &'s Settings> for StatsTarget {
    fn target_type(&self) -> TargetType {
        TargetType::Stats
    }
    fn export(
        &self,
        root: &Element,
        settings: &'s Settings,
        args: &'a StatsArgs,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let mut stats = Stats::default();

        stats.line_count = root.get_position().end.line - 1;
        stats.run(root, (settings, args), out)?;

        writeln!(
            out,
            "{}",
            serde_json::to_string(&stats).expect("could not serialize the stats struct")
        )
    }
}
