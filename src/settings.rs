use util::TravFunc;
use std::collections::HashMap;
use config;

pub type Settings = config::Config;

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

# A list of lowercase template name prefixes which will be stripped if found.
template_prefixes:
    - \":mathe für nicht-freaks: vorlage:\"
    - file
    - datei
    - bild

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
    # extension of the resulting file. Used for make dependency generation.
    target_extension: \"yml\"
    # are dependencies generated for this target?
    generate_deps: false
    # mapping of external file extensions to target extensions.
    # this is useful if external dependencies should be processed by
    # make for this target.
    deps_extension_mapping: {}
";


/// An export target.
pub struct Target<'a, 'b: 'a, 'c: 'a> {
    /// The target name.
    pub name: String,
    /// A function to call for export.
    pub export_func: &'a TravFunc<'c, &'b config::Config>,
    /// mfnf transformations applied?
    pub with_transformation: bool,
}

pub fn default_config() -> config::Config {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::from_str(DEFAULT_SETTINGS, config::FileFormat::Yaml))
        .expect("config parse error!");
    settings
}

#[macro_export]
macro_rules! setting {
    ($settings:ident $( . $attr:ident)+) => {{
        let mut path = vec![$(stringify!($attr)),*];
        let thing = path.remove(0);
        let mut base: config::Value = $settings.get(thing)
            .expect(&format!("missing setting: {}", &thing));

        while path.len() > 0 {
            let thing = path.remove(0);
            base = base.into_table()
                .expect(&format!("attribute error: {}", &thing))
                .remove(thing)
                .expect(&format!("attribute error: {}", &thing));
        }
        base.try_into().expect("wrong setting type!")
    }}
}

#[macro_export]
macro_rules! from_table {
    ($settings:ident $( . $attr:ident)+) => {{
        let mut path = vec![$(stringify!($attr)),*];
        let mut base: config::Value = $settings.clone();
        while path.len() > 0 {
            let thing = path.remove(0);
            base = base.into_table()
                .expect(&format!("attribute error: {}", &thing))
                .remove(thing)
                .expect(&format!("attribute error: {}", &thing));
        }
        base.try_into().expect("wrong setting type!")
    }}
}

pub fn target_settings<'a>(config: &'a config::Config,
                       target: &str) -> Option<HashMap<String,config::Value>> {

    let targets = config.get_array("targets")
        .expect("Settings object does not specify targets!");
    for raw_target in targets {
        let target_map = raw_target.into_table().unwrap();
        let name = target_map.keys().next().unwrap();
        if name == target {
            return Some(target_map.get(name).unwrap().clone().into_table().unwrap())
        }
    }
    return None
}

