use mfnf_sitemap::Markers;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use anchors;
use compose;
use deps;
use html;
use latex;
use normalize;
use pdf;
use sections;
use stats;
use MFNFTargets;

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
    /// The targets defined on the settings.
    /// Maps a target name to a target definition.
    /// This allows for multiple targets of the same type with different parameters.
    pub targets: HashMap<String, MFNFTargets>,

    /// A list of file prefixes which indicate references to files.
    pub file_prefixes: Vec<String>,

    /// Base path for web links to articles.
    pub article_url_base: String,

    /// Path base for dependency writing accessed at run time.
    pub base_path: PathBuf,

    /// Path to embedded media files. (relative to `media_path`)
    pub media_path: PathBuf,

    /// Path to the section file directory. (relative to `section_path`)
    pub section_path: PathBuf,

    /// Default revision number of included sections (always `latest`)
    pub section_rev: String,

    /// File extensions for section files
    pub section_ext: String,

    /// Template name prefix indication section inclusion
    pub section_inclusion_prefix: String,

    /// Mapping of interwiki link prefix to url (e.g. w: -> de.wikipedia.org)
    pub interwiki_link_mapping: HashMap<String, String>,

    /// Caption text used in a reference to an anchor. (usually localized)
    /// This is important for matching internal references to link targets (anchors).
    pub anchor_caption: String,
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
                    "section-deps".to_string(),
                    MFNFTargets::SectionDeps(deps::SectionDepsTarget::default()),
                );
                tmap.insert(
                    "media-deps".to_string(),
                    MFNFTargets::MediaDeps(deps::MediaDepsTarget::default()),
                );
                tmap.insert(
                    "html".to_string(),
                    MFNFTargets::HTML(html::HTMLTarget::default()),
                );
                tmap.insert(
                    "latex".to_string(),
                    MFNFTargets::Latex(latex::LatexTarget::default()),
                );
                tmap.insert(
                    "sections".to_string(),
                    MFNFTargets::Sections(sections::SectionsTarget::default()),
                );
                tmap.insert(
                    "pdf".to_string(),
                    MFNFTargets::PDF(pdf::PDFTarget::default()),
                );
                tmap.insert(
                    "stats".to_string(),
                    MFNFTargets::Stats(stats::StatsTarget::default()),
                );
                tmap.insert(
                    "anchors".to_string(),
                    MFNFTargets::Anchors(anchors::AnchorsTarget::default()),
                );
                tmap.insert(
                    "normalize".to_string(),
                    MFNFTargets::Normalize(normalize::NormalizeTarget::default()),
                );
                tmap.insert(
                    "compose".to_string(),
                    MFNFTargets::Compose(compose::ComposeTarget::default()),
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
            base_path: ".".into(),
            media_path: "media".into(),
            article_url_base: "https://de.wikibooks.org/wiki/".into(),
            section_path: "sections".into(),
            section_rev: "latest".into(),
            section_ext: "json".into(),
            section_inclusion_prefix: "#lst:".into(),
            anchor_caption: "Anker".into(),
        }
    }
}
