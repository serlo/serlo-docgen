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
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LatexTarget {
    /// mapping of external file extensions to target extensions.
    /// this is useful if external dependencies should be processed by
    /// make for this target.
    deps_extension_mapping: HashMap<String, String>,

    /// Indentation depth for template content.
    indentation_depth: usize,
    /// Maximum line width (without indentation).
    max_line_width: usize,
    /// Specifies how many images a gallery may have on one row.
    gallery_images_per_row: usize,
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
            deps_extension_mapping: string_map![
                "png" => "png.pdf",
                "svg" => "svg.pdf",
                "eps" => "eps.pdf",
                "jpg" => "jpg.pdf",
                "jpeg" => "jpg.pdf",
                "gif" => "gif.qr.pdf",
                "webm" => "webm.qr.pdf",
                "mp4" => "mp4.qr.pdf"
            ],
            indentation_depth: 4,
            max_line_width: 80,
            image_width: 0.5,
            image_height: 0.2,
            gallery_images_per_row: 2,
            environments: string_value_map![
                "definition" => string_vec!["definition"],
                "example" => string_vec!["example"],
                "proofbycases" => string_vec!["cases", "proofs"],
                "solution" => string_vec!["solution"],
                "solutionprocess" => string_vec!["solutionprocess"],
                "proofsummary" => string_vec!["proofsummary"],
                "alternativeproof" => string_vec!["alternativeproof"],
                "proof" => string_vec!["proof"],
                "warning" => string_vec!["warning"],
                "hint" => string_vec!["hint"],
                "question" => string_vec!["question", "answer", "questiontype"],
                "theorem" => string_vec!["theorem", "explanation", "example",
                                         "proofsummary", "solutionprocess", "solution",
                                         "proof"],
                "proofsummary" => string_vec!["proofsummary"],
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

    fn do_include_sections(&self) -> bool { true }
    fn get_target_extension(&self) -> &str { "tex" }
    fn get_extension_mapping(&self) -> &HashMap<String, String> {
        &self.deps_extension_mapping
    }
    fn export<'a>(&self,
                  root: &'a Element,
                  settings: &Settings,
                  _: &[String],
                  out: &mut io::Write) -> io::Result<()> {

        // apply latex-specific transformations
        let mut latex_tree = root.clone();
        latex_tree = trans::normalize_formula(latex_tree, settings)
            .expect("Error in formula normalization!");
        latex_tree = trans::hoist_thumbnails(latex_tree, settings)
            .expect("Error in thumbnail hoisting!");

        let mut renderer = LatexRenderer::new(self);
        renderer.run(&latex_tree, settings, out)
    }
}




