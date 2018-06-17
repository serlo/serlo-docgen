use mfnf_sitemap::Markers;
use std::collections::HashMap;
use std::path::PathBuf;

use deps;
use html;
use latex;
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

    /// Path prefix for external files.
    pub external_file_path: PathBuf,

    /// Path to the section file directory.
    pub section_path: PathBuf,

    /// Default revision number of included sections (always `latest`)
    pub section_rev: String,

    /// File extensions for section files
    pub section_ext: String,

    /// Template name prefix indication section inclusion
    pub section_inclusion_prefix: String,
}

impl Default for RuntimeSettings {
    fn default() -> RuntimeSettings {
        RuntimeSettings {
            document_title: "<no document name specified>".into(),
            document_revision: "latest".into(),
            tex_checker: None,
            markers: Markers::default(),
            target_name: "".into(),
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
                tmap
            },
            file_prefixes: string_vec!["file:", "datei:", "bild:"],
            external_file_path: "media".into(),
            article_url_base: "https://de.wikibooks.org/wiki/".into(),
            section_path: "sections".into(),
            section_rev: "latest".into(),
            section_ext: "yml".into(),
            section_inclusion_prefix: "#lst:".into(),
        }
    }
}
