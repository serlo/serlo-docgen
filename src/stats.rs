//! Implements the `stats` target which extracts various statistical
//! information from the document tree.

use std::collections::HashMap;
use preamble::*;

use std::io;
use serde_yaml;


/// Dump stats to stdout as yaml.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct StatsTarget {
    #[serde(skip_serializing_if = "is_default")]
    pub extension_mapping: HashMap<String, String>,
}

impl Default for StatsTarget {
    fn default() -> StatsTarget {
        StatsTarget {
            extension_mapping: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
struct Stats {
    // test stat counting the document length
    pub line_count: usize,
}

impl Target for StatsTarget {
    fn do_include_sections(&self) -> bool { false }
    fn get_target_extension(&self) -> &str { "yml" }
    fn get_extension_mapping(&self) -> &HashMap<String, String> {
        &self.extension_mapping
    }
    fn export<'a>(&self,
                root: &'a Element,
                _settings: &Settings,
                _args: &[String],
                out: &mut io::Write) -> io::Result<()> {

        let mut stats = Stats {
            line_count: 0,
        };

        stats.line_count = root.get_position().end.line;

        writeln!(out, "{}", serde_yaml::to_string(&stats)
            .expect("could not serialize the stats struct"))
    }
}
