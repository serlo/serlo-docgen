use mfnf_sitemap::Markers;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::{
    AnchorsTarget, ComposeTarget, HTMLTarget, LatexTarget, MediaDepTarget, NormalizeTarget,
    PDFTarget, SectionDepTarget, SectionsTarget, StatsTarget, Targets,
};

use mwparser_utils::CachedTexChecker;

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

/// MFNF transformation settings object.
#[derive(Debug, Default)]
pub struct Settings {
    pub runtime: RuntimeSettings,
    pub general: GeneralSettings,
}

/// Runtime (instance-specific) settings.
#[derive(Debug)]
pub struct RuntimeSettings {
    /// Currently used text checker
    pub tex_checker: Option<CachedTexChecker>,

    /// Article markers (excludes / includes)
    pub markers: Markers,

    /// Title of the current document.
    pub document_title: String,

    /// Additional revision id of an article.
    pub document_revision: String,

    /// The current target name (index into defined targets).
    pub target_name: String,

    /// A list of internal reference URLs available in this export.
    ///
    /// If an internal reference is not in this list, it shall be included
    /// as a hyperlink to the external source.
    pub available_anchors: HashSet<String>,
}

/// General MFNF transformation settings for all targets.
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct GeneralSettings {
    /// Mapping of a target configuration name to a list of configured targets
    /// belonging to the semantic target configuration class.
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

impl Default for RuntimeSettings {
    fn default() -> RuntimeSettings {
        RuntimeSettings {
            document_title: "<no document name specified>".into(),
            document_revision: "latest".into(),
            tex_checker: None,
            markers: Markers::default(),
            target_name: "".into(),
            available_anchors: HashSet::new(),
        }
    }
}

impl Default for GeneralSettings {
    fn default() -> GeneralSettings {
        GeneralSettings {
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
