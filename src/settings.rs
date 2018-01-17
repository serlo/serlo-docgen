use util::TravFunc;
use std::collections::HashMap;


/// An export target.
pub struct Target<'a> {
    /// The target name.
    pub name: String,
    /// Target export settings.
    pub settings: Settings,
    /// The path to write output files to.
    pub output_path: String,
    /// A function to call for export.
    pub export_func: &'a TravFunc<'a, &'a Settings>,
    /// Does this target operate on the input tree directly or with
    /// mfnf transformations applied?
    pub with_transformation: bool,
}

/// General MFNF transformation settings for all targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub download_images: bool,
    pub latex_settings: LaTeXSettings,
    pub deps_settings: DepSettings,

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
}

/// General MFNF transformation settings for all targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaTeXSettings {

    /// Page trim in mm.
    pub page_trim: f32,
    /// Paper width in mm.
    pub page_width: f32,
    /// Paper height in mm.
    pub page_height: f32,
    /// Font size in pt.
    pub font_size: f32,
    /// Baseline height in pt.
    pub baseline_height: f32,
    /// Paper border in mm as [top, bottom, outer, inner]
    pub border: [f32; 4],
    /// Document class options.
    pub document_options: String,
    /// Indentation depth for template content.
    pub indentation_depth: usize,
    /// Maximum line widht (without indentation).
    pub max_line_width: usize,
    /// Maximum width of an image in a figure as fraction of \textwidth
    pub image_width: f32,
    /// Maximum height of an imgae in a figure as fraction of \textheight
    pub image_height: f32,

    /// Templates which can be exported as an environment.
    /// The template may have a `title` attribute and a content
    /// attribute, which has the same name as the environment.
    /// Any additional template attributes will be exported as
    /// subsequent environments.
    pub environments: HashMap<String, Vec<String>>,
}


/// Settings for article dependencies target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepSettings {
    /// File extensions indicaing images.
    pub image_extensions: Vec<String>,
    /// Path prefix for images.
    pub image_path: String,
    /// Path to the section file directory.
    pub section_path: String,
    /// Revision number of included sections (always "latest")
    pub section_rev: String,
    /// File extensions for section files
    pub section_ext: String,
    /// Template name prefix indication section inclusion
    pub section_inclusion_prefix: String
}


macro_rules! s {
    ($str:expr) => {
        String::from($str)
    };
    ($s1:expr, $s2:expr) => {
        (String::from($s1), String::from($s2))
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            download_images: true,
            latex_settings: LaTeXSettings::default(),
            deps_settings: DepSettings::default(),
            document_title: s!("<no document name specified>"),
            document_revision: s!("latest"),
            file_prefixes: vec![s!("file"), s!("datei"), s!("bild")],
            translations: [
                s!("beispiel", "example"),
                s!("definition", "definition"),
                s!("satz", "theorem"),
                s!("lösung", "solution"),
                s!("lösungsweg", "solutionprocess"),
                s!("titel", "title"),
                s!("formel", "formula"),
                s!("fallunterscheidung", "proofbycases"),
                s!("fall_list", "cases"),
                s!("beweis_list", "proofs"),
                s!("beweiszusammenfassung", "proofsummary"),
                s!("alternativer beweis", "alternativeproof"),
                s!("beweis", "proof"),
                s!("warnung", "warning"),
                s!("hinweis", "hint"),
                s!("frage", "question"),
                s!("antwort", "answer"),
            ].iter().cloned().collect(),
            template_prefixes: vec![s!(":mathe für nicht-freaks: vorlage:")],
        }
    }
}

impl Default for LaTeXSettings {
    fn default() -> Self {
        LaTeXSettings {
            page_trim: 0.,
            page_width: 155.,
            page_height: 235.,
            font_size: 9.,
            baseline_height: 12.,
            border: [20.5, 32.6, 22., 18.5],
            indentation_depth: 4,
            max_line_width: 80,
            image_width: 0.5,
            image_height: 0.2,
            document_options: String::from("tocflat, listof=chapterentry"),
            environments: [
                (s!("definition"),          vec![s!("definition")]),
                (s!("theorem"),             vec![s!("theorem"), s!("explanation"),
                                                 s!("example"), s!("proofsummary"),
                                                 s!("solutionprocess"), s!("solution"),
                                                 s!("proof")
                                            ]),
                (s!("solution"),            vec![s!("solution")]),
                (s!("solutionprocess"),     vec![s!("solutionprocess")]),
                (s!("proof"),               vec![s!("proof")]),
                (s!("proofsummary"),        vec![s!("proofsummary")]),
                (s!("alternativeproof"),    vec![s!("alternativeproof")]),
                (s!("hint"),                vec![s!("hint")]),
                (s!("warning"),             vec![s!("warning")]),
                (s!("example"),             vec![s!("example")]),
                (s!("importantparagraph"),  vec![s!("importantparagraph")]),
                (s!("exercise"),            vec![s!("theorem"), s!("explanation"),
                                                 s!("example"), s!("proofsummary"),
                                                 s!("solutionprocess"), s!("solution"),
                                                 s!("proof")
                                            ]),
                (s!("explanation"),         vec![s!("explanation")]),
            ].iter().cloned().collect(),
        }
    }
}

impl Default for DepSettings {
    fn default() -> Self {
        DepSettings {
            image_extensions: vec![
                s!("jpg"),
                s!("jpeg"),
                s!("png"),
                s!("gif"),
                s!("svg"),
                s!("eps"),
                s!("pdf"),
            ],
            image_path: s!("images"),
            section_path: s!("sections"),
            section_rev: s!("latest"),
            section_ext: s!("yml"),
            section_inclusion_prefix: s!("#lst:"),
        }
    }
}
