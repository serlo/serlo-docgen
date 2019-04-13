use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    AnchorsTarget, ComposeTarget, HTMLTarget, LatexTarget, MediaDepTarget, NormalizeTarget,
    PDFTarget, SectionDepTarget, SectionsTarget, SerloTarget, StatsTarget, Targets,
};

macro_rules! string_vec {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

macro_rules! string_value_map {
    ($($k:expr => $v:expr),*) => {{
        let mut map = HashMap::new();
        $(map.insert($k.to_string(), $v);)*
        map
    }}
}

/// General MFNF transformation settings for all targets.
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    /// Mapping of a target configuration (subtarget) name to a list configured targets belonging
    /// to this subtarget class.
    pub targets: HashMap<String, Vec<Targets>>,

    /// A list of file prefixes which indicate references to files.
    pub file_prefixes: Vec<String>,

    /// Base path for web links to articles.
    pub article_url_base: String,

    /// Path to embedded media files. (relative to `media_path`)
    pub media_path: PathBuf,

    /// Mapping of interwiki link prefix to url (e.g. w: -> de.wikipedia.org)
    pub interwiki_link_mapping: HashMap<String, String>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            targets: {
                let mut tmap = HashMap::new();
                tmap.insert(
                    "default".to_string(),
                    vec![
                        Targets::Sections(SectionsTarget::default()),
                        Targets::SectionDeps(SectionDepTarget::default()),
                        Targets::MediaDeps(MediaDepTarget::default()),
                        Targets::Normalize(NormalizeTarget::default()),
                        Targets::Compose(ComposeTarget::default()),
                        Targets::Anchors(AnchorsTarget::default()),
                        Targets::Latex(LatexTarget::default()),
                        Targets::PDF(PDFTarget::default()),
                        Targets::Stats(StatsTarget::default()),
                        Targets::HTML(HTMLTarget::default()),
                        Targets::Serlo(SerloTarget::default()),
                    ],
                );
                tmap
            },
            interwiki_link_mapping: [
                ("w:", "https://de.wikipedia.org/wiki/"),
                ("b:", "https://de.wikibooks.org/wiki/"),
                ("d:", "https://www.wikidata.org/wiki/"),
                ("n:", "https://de.wikinews.org/wiki/"),
                ("v:", "https://de.wikiversity.org/wiki/"),
                ("wikt:", "https://de.wiktionary.org/wiki/"),
            ]
            .iter()
            .map(|e| (e.0.to_string(), e.1.to_string()))
            .collect(),
            file_prefixes: string_vec!["file:", "datei:", "bild:"],
            media_path: "media".into(),
            article_url_base: "https://de.wikibooks.org/wiki/".into(),
        }
    }
}
