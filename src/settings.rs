use mediawiki_parser::ast::Element;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::io;
use deps;
use latex;
use MFNFTargets;

pub const DEFAULT_SETTINGS: &'static str = "
# Title of the current document.
document_title: \"<no document title>\"

# Additional revision id of an article.
document_revision: latest

# Maps a template names and template attribute names to their translations.
# E.g. german template names to their englisch translations.
translations:
    beispiel: example
    definition: definition
    satz: theorem
    lösung: solution
    lösungsweg: solutionprocess
    titel: title
    formel: formula
    fallunterscheidung: proofbycases
    fall_list: cases
    beweis_list: proofs
    beweiszusammenfassung: proofsummary
    alternativer beweis: alternativeproof
    beweis: proof
    warnung: warning
    hinweis: hint
    frage: question
    antwort: answer
    anker: anchor

# A list of lowercase template name prefixes which will be stripped if found.
template_prefixes:
    - \":mathe für nicht-freaks: vorlage:\"

# A list of file prefixes which are ignored.
file_prefixes:
    - file
    - datei
    - bild

# Target - specific settings
targets:
  latex:
    # Does this target operate on the input tree directly or with
    # mfnf transformations applied?
    with_transformation: true
    # extension of the resulting file. Used for make dependency generation.
    target_extension: \"tex\"
    # are dependencies generated for this target?
    generate_deps: true
    # mapping of external file extensions to target extensions.
    # this is useful if external dependencies should be processed by
    # make for this target.
    deps_extension_mapping:
        png: pdf
        svg: pdf
        eps: pdf
        jpg: pdf
        jpeg: pdf
        gif: pdf

    # Page trim in mm.
    page_trim: 0.0
    # Paper width in mm.
    page_width: 155.0
    # Paper height in mm.
    page_height: 235.0
    # Font size in pt.
    font_size: 9.0
    # Baseline height in pt.
    baseline_height: 12.0
    # Paper border in mm as [top, bottom, outer, inner]
    border: [20.5, 32.6, 22.0, 18.5]
    # Document class options.
    document_options: >
        tocflat, listof=chapterentry
    # Indentation depth for template content.
    indentation_depth: 4
    # Maximum line width (without indentation).
    max_line_width: 80
    # Maximum width of an image in a figure as fraction of \\textwidth
    image_width: 0.5
    # Maximum height of an imgae in a figure as fraction of \\textheight
    image_height: 0.2

    # Templates which can be exported as an environment.
    # The template may have a `title` attribute and a content
    # attribute, which has the same name as the environment.
    # Any additional template attributes will be exported as
    # subsequent environments, if listed here.
    environments:
        definition:
            - definition
        theorem:
            - theorem
            - explanation
            - example
            - proofsummary
            - solutionprocess
            - solution
            - proof
        solution:
            - solution
        solutionprocess:
            - solutionprocess
        proof:
            - proof
        proofsummary:
            - proofsummary
        alternativeproof:
            - alternativeproof
        hint:
            - hint
        warning:
            - warning
        example:
            - example
        importantparagraph:
            - importantparagraph
        exercise:
            - theorem
            - explanation
            - example
            - proofsummary
            - solutionprocess
            - solution
            - proof
            - exercise
        explanation:
            - explanation
  deps:
    # Does this target operate on the input tree directly or with
    # mfnf transformations applied?
    with_transformation: false
    # extension of the resulting file. Used for make dependency generation.
    target_extension: \"dep\"
    # are dependencies generated for this target?
    generate_deps: false
    # mapping of external file extensions to target extensions.
    # this is useful if external dependencies should be processed by
    # make for this target.
    deps_extension_mapping: {}

    # File extensions indicating images.
    image_extensions:
        - jpg
        - jpeg
        - png
        - gif
        - svg
        - eps
        - pdf
    # Path prefix for images.
    image_path: images
    # Path to the section file directory.
    section_path: sections
    # Revision number of included sections (always `latest`)
    section_rev: latest
    # File extensions for section files
    section_ext: yml
    # Template name prefix indication section inclusion
    section_inclusion_prefix: \"#lst:\"
  sections:
    # Does this target operate on the input tree directly or with
    # mfnf transformations applied?
    with_transformation: false

    target_extension: \"yml\"
    # are dependencies generated for this target?
    generate_deps: false
    # mapping of external file extensions to target extensions.
    # this is useful if external dependencies should be processed by
    # make for this target.
    deps_extension_mapping: {}
";


#[macro_export]
macro_rules! string_vec {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

#[macro_export]
macro_rules! string_map {
    ($($k:expr => $v:expr),*) => {{
        let mut map: HashMap<String, String> = HashMap::new();
        $(map.insert($k.to_string(), $v.to_string());)*
        map
    }}
}

#[macro_export]
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
            section_path: "section".into(),
            section_rev: "latest".into(),
            section_ext: "yml".into(),
            section_inclusion_prefix: "#lst:".into(),
        }
    }
}

