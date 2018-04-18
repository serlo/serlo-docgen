use std::collections::HashMap;

use deps;
use latex;
use sections;
use pdf;
use MFNFTargets;

use mfnf_commons::util::CachedTexChecker;
use serde::ser::{Serialize, Serializer, SerializeStruct};

macro_rules! string_vec {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

macro_rules! string_map {
    ($($k:expr => $v:expr),*) => {{
        let mut map: HashMap<String, String> = HashMap::new();
        $(map.insert($k.to_string(), $v.to_string());)*
        map
    }}
}

macro_rules! string_value_map {
    ($($k:expr => $v:expr),*) => {{
        let mut map = HashMap::new();
        $(map.insert($k.to_string(), $v);)*
        map
    }}
}


/// General MFNF transformation settings for all targets.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The targets defined on the settings.
    /// Maps a target name to a target definition.
    /// This allows for multiple targets of the same type with different parameters.
    pub targets: HashMap<String, MFNFTargets>,

    /// Title of the current document.
    pub document_title: String,

    /// Additional revision id of an article.
    pub document_revision: String,

    /// A list of file prefixes which are ignored.
    pub file_prefixes: Vec<String>,

    /// File extensions allowed for references to external files.
    pub external_file_extensions: Vec<String>,

    /// Base path for web links to articles.
    pub article_url_base: String,

    /// Path prefix for external files.
    pub external_file_path: String,

    /// Path to the section file directory.
    pub section_path: String,

    /// Default revision number of included sections (always `latest`)
    pub section_rev: String,

    /// File extensions for section files
    pub section_ext: String,

    /// Template name prefix indication section inclusion
    pub section_inclusion_prefix: String,

    /// Currently used text checker
    #[serde(skip, default)]
    pub tex_checker: Option<CachedTexChecker>,
}

impl Serialize for Settings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let default = Settings::default();

        let mut state = serializer.serialize_struct("Settings", 10)?;
        state.serialize_field("targets", &self.targets)?;
        ser_field_non_default!(self, document_title, default, state);
        ser_field_non_default!(self, document_revision, default, state);
        ser_field_non_default!(self, file_prefixes, default, state);
        ser_field_non_default!(self, external_file_extensions, default, state);
        ser_field_non_default!(self, article_url_base, default, state);
        ser_field_non_default!(self, external_file_path, default, state);
        ser_field_non_default!(self, section_path, default, state);
        ser_field_non_default!(self, section_rev, default, state);
        ser_field_non_default!(self, section_ext, default, state);
        ser_field_non_default!(self, section_inclusion_prefix, default, state);
        state.end()
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            targets: {
                let mut tmap = HashMap::new();
                tmap.insert("deps".to_string(),
                    MFNFTargets::Dependencies(deps::DepsTarget::default()));
                tmap.insert("latex".to_string(),
                    MFNFTargets::Latex(latex::LatexTarget::default()));
                tmap.insert("sections".to_string(),
                    MFNFTargets::Sections(sections::SectionsTarget::default()));
                tmap.insert("pdf".to_string(),
                    MFNFTargets::PDF(pdf::PDFTarget::default()));
                tmap
            },
            document_title: "<no document name specified>".into(),
            document_revision: "latest".into(),
            file_prefixes: string_vec!["file", "datei", "bild"],
            external_file_extensions: string_vec![
                "jpg",
                "jpeg",
                "png",
                "svg",
                "eps",
                "pdf",
                "gif",
                "webm",
                "mp4"
            ],
            external_file_path: "media".into(),
            article_url_base: "https://de.wikibooks.org/wiki/".into(),
            section_path: "sections".into(),
            section_rev: "latest".into(),
            section_ext: "yml".into(),
            section_inclusion_prefix: "#lst:".into(),
            tex_checker: None,
        }
    }
}

