use std::collections::HashMap;
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

    /// Base path for web links to articles.
    pub article_url_base: String,

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

    /// Path to the texvccheck executable
    pub texvccheck_path: String,

    /// Whether Tex formulas should be checked with texvccheck
    pub check_tex_formulas: bool,
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
                "antwort" => "answer",
                "anker" => "anchor",
                "liste" => "list"
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
            article_url_base: "https://de.wikibooks.org/wiki/".into(),
            section_path: "sections".into(),
            section_rev: "latest".into(),
            section_ext: "yml".into(),
            section_inclusion_prefix: "#lst:".into(),
            texvccheck_path: "mk/bin/texvccheck".into(),
            check_tex_formulas: false,
        }
    }
}

