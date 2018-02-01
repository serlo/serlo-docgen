//! Implementation of the `latex` target.
//!
//! This target renders the final syntax tree to a LaTeX document body.
//! LaTeX boilerplate like preamble or document tags have to be added afterwards.

use std::collections::HashMap;
use preamble::*;

mod trans;
mod renderer;

use self::renderer::{LatexRenderer};

/// Data for LaTeX export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatexTarget {
    /// Does this target operate on the input tree directly or with
    /// mfnf transformations applied?
    with_transformation: bool,
    /// extension of the resulting file. Used for make dependency generation.
    target_extension: String,
    /// are dependencies generated for this target?
    generate_deps: bool,
    /// mapping of external file extensions to target extensions.
    /// this is useful if external dependencies should be processed by
    /// make for this target.
    deps_extension_mapping: HashMap<String, String>,

    /// Page trim in mm.
    page_trim: f32,
    /// Paper width in mm.
    page_width: f32,
    /// Paper height in mm.
    page_height: f32,
    /// Font size in pt.
    font_size: f32,
    /// Baseline height in pt.
    baseline_height: f32,
    /// Paper border in mm as [top, bottom, outer, inner]
    border: [f32; 4],
    /// Document class options.
    document_options: String,
    /// Indentation depth for template content.
    indentation_depth: usize,
    /// Maximum line width (without indentation).
    max_line_width: usize,
    /// Maximum width of an image in a figure as fraction of \\textwidth
    image_width: f32,
    /// Maximum height of an imgae in a figure as fraction of \\textheight
    image_height: f32,

    /// Templates which can be exported as an environment.
    /// The template may have a `title` attribute and a content
    /// attribute, which has the same name as the environment.
    /// Any additional template attributes will be exported as
    /// subsequent environments, if listed here.
    environments: HashMap<String, Vec<String>>,
}

impl Default for LatexTarget {
    fn default() -> LatexTarget {
        LatexTarget {
            with_transformation: true,
            target_extension: "tex".into(),
            generate_deps: true,
            deps_extension_mapping: string_map![
                "png" => "pdf",
                "svg" => "pdf",
                "eps" => "pdf",
                "jpg" => "pdf",
                "jpeg" => "pdf",
                "gif" => "pdf"
            ],
            page_trim: 0.0,
            page_width: 155.0,
            page_height: 235.0,
            font_size: 9.0,
            baseline_height: 12.0,
            border: [20.5, 32.6, 22.0, 18.5],
            document_options: "tocflat, listof=chapterentry".into(),
            indentation_depth: 4,
            max_line_width: 80,
            image_width: 0.5,
            image_height: 0.2,
            environments: string_value_map![
                "definition" => string_vec!["definition"],
                "theorem" => string_vec!["theorem", "explanation", "example",
                                         "proofsummary", "solutionprocess", "solution",
                                         "proof"],
                "solution" => string_vec!["solution"],
                "solutionprocess" => string_vec!["solutionprocess"],
                "proof" => string_vec!["proof"],
                "proofsummary" => string_vec!["proofsummary"],
                "alternativeproof" => string_vec!["alternativeproof"],
                "hint" => string_vec!["hint"],
                "warning" => string_vec!["warning"],
                "example" => string_vec!["example"],
                "importantparagraph" => string_vec!["importantparagraph"],
                "exercise" => string_vec!["exercise", "explanation", "example",
                                          "proofsummary", "solutionprocess", "solution",
                                          "proof"],
                "explanation" => string_vec!["explanation"]
            ],
        }
    }
}

impl Target for LatexTarget {

    fn get_name(&self) -> &str { "latex" }
    fn do_include_sections(&self) -> bool { true }
    fn do_generate_dependencies(&self) -> bool { true }
    fn get_target_extension(&self) -> &str { "tex" }
    fn get_extension_mapping(&self) -> &HashMap<String, String> {
        &self.deps_extension_mapping
    }
    fn export<'a>(&self,
                  root: &'a Element,
                  settings: &Settings,
                  out: &mut io::Write) -> io::Result<()> {

        // apply latex-specific transformations
        let mut latex_tree = root.clone();
        latex_tree = trans::normalize_formula(latex_tree, settings)
            .expect("Could not appy LaTeX-Secific transformations!");

        let mut renderer = LatexRenderer::new(self);
        renderer.run(&latex_tree, settings, out)
    }
}




