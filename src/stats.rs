//! Implements the `stats` target which extracts various statistical
//! information from the document tree.
use std::collections::HashMap;
use preamble::*;

use std::io;
use serde_yaml;


/// Dump stats to stdout as yaml.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct StatsTarget {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
struct Stats<'e> {
    #[serde(skip)]
    pub path: Vec<&'e Element>,

    /// The original document length
    pub line_count: usize,

    /// Number of images included
    pub image_count: usize,

    /// Number of templates used of a kind
    pub template_count: HashMap<String, usize>,
}


impl<'e, 's: 'e> Traversion<'e, &'s Settings> for Stats<'e> {

    path_methods!('e);

    fn work(
        &mut self,
        root: &Element,
        settings: &'s Settings,
        _out: &mut io::Write
    ) -> io::Result<bool> {

        match root {
            Element::InternalReference(ref iref) => {
                let is_image = settings.general.external_file_extensions
                    .iter().any(|suffix|
                        extract_plain_text(&iref.target)
                        .trim()
                        .to_lowercase()
                        .ends_with(suffix)
                    );
                if is_image {
                    self.image_count += 1
                }
            },
            Element::Template(ref template) => {
                let name = extract_plain_text(&template.name).trim().to_lowercase();
                let current = *self.template_count.get(&name).unwrap_or(&0);
                self.template_count.insert(name.clone(), current + 1);
            },
            _ => (),
        };
        Ok(true)
    }
}

impl Target for StatsTarget {
    fn include_sections(&self) -> bool { true }
    fn target_extension(&self) -> &str { "yml" }
    fn extension_for(&self, _ext: &str) -> &str { "%" }
    fn export<'a>(&self,
                root: &'a Element,
                settings: &Settings,
                _args: &[String],
                out: &mut io::Write) -> io::Result<()> {

        let mut stats = Stats::default();

        stats.line_count = root.get_position().end.line - 1;
        stats.run(root, settings, out)?;

        writeln!(out, "{}", serde_yaml::to_string(&stats)
            .expect("could not serialize the stats struct"))
    }
}
