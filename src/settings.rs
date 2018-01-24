use mediawiki_parser::ast::Element;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::io;
use deps;
use latex;
use sections;
use MFNFTargets;

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

pub trait Target {
    /// export the the ast to `out`.
    fn export<'a>(&self,
                  root: &'a Element,
                  path: &mut Vec<&'a Element>,
                  settings: &Settings,
                  out: &mut io::Write) -> io::Result<()>;
    /// get the name of this target.
    fn get_name(&self) -> &str;
    /// does this target operate on the input tree directly or with
    /// mfnf transformations applied?
    fn do_include_sections(&self) -> bool { false }
    /// are make dependencies generated for this target?
    fn do_generate_dependencies(&self) -> bool { false }
    /// extension of the resulting file. Used for make dependency generation.
    fn get_target_extension(&self) -> &str;
    /// mapping of external file extensions to target extensions.
    /// this is useful if external dependencies should be processed by
    /// make for this target.
    fn get_extension_mapping(&self) -> &HashMap<String, String>;
}



/// General MFNF transformation settings for all targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// The targets defined on the settings.
    /// Maps a target name to a target definition.
    /// This allows for multiple targets of the same type with different parameters.
    pub targets: HashMap<String, MFNFTargets>,

    /// Title of the current document.
    pub document_title: String,

    /// Additional revision id of an article.
    pub document_revision: String,

    /// Maps a template names and template attribute names to their translations.
    /// E.g. german template names to their englisch translations.
    pub translations: HashMap<String, String>,

    /// A list of lowercase template name prefixes which will be stripped if found.
    pub template_prefixes: Vec<String>,

    /// A list of file prefixes which are ignored.
    pub file_prefixes: Vec<String>,

    /// File extensions indicating images.
    pub image_extensions: Vec<String>,

    /// Path prefix for images.
    pub image_path: String,

    /// Path to the section file directory.
    pub section_path: String,

    /// Default revision number of included sections (always `latest`)
    pub section_rev: String,

    /// File extensions for section files
    pub section_ext: String,

    /// Template name prefix indication section inclusion
    pub section_inclusion_prefix: String,
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
                tmap
            },
            document_title: "<no document name specified>".into(),
            document_revision: "latest".into(),
            file_prefixes: string_vec!["file", "datei", "bild"],
            translations: string_map![
                "beispiel" => "example",
                "definition" => "definition",
                "satz" => "theorem",
                "lösung" => "solution",
                "lösungsweg" => "solutionprocess",
                "titel" => "title",
                "formel" => "formula",
                "fallunterscheidung" => "proofbycases",
                "fall_list" => "cases",
                "beweis_list" => "proofs",
                "beweiszusammenfassung" => "proofsummary",
                "alternativer beweis" => "alternativeproof",
                "beweis" => "proof",
                "warnung" => "warning",
                "hinweis" => "hint",
                "frage" => "question",
                "antwort" => "answer"
            ],
            template_prefixes: string_vec![":mathe für nicht-freaks: vorlage:"],
            image_extensions: string_vec!["jpg",
                                            "jpeg",
                                            "png",
                                            "gif",
                                            "svg",
                                            "eps",
                                            "pdf"],
            image_path: "images".into(),
            section_path: "sections".into(),
            section_rev: "latest".into(),
            section_ext: "yml".into(),
            section_inclusion_prefix: "#lst:".into(),
        }
    }
}

