//! Implementation of the `latex` target.
//!
//! This target renders the final syntax tree to a LaTeX document body.
//! LaTeX boilerplate like preamble or document tags have to be added afterwards.

use crate::preamble::*;
use crate::transformations;
use std::collections::{HashMap, HashSet};

mod renderer;

use self::renderer::LatexRenderer;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct LatexArgs {
    /// Title of the document beeing processed.
    document_title: String,

    /// Path to a list of link targets (anchors) available in the export.
    #[structopt(parse(try_from_str = "load_anchor_set"))]
    available_anchors: HashSet<String>,
}

/// Data for LaTeX export.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct LatexTarget {
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
    /// Show caption for embedded images?
    centered_image_captions: bool,
    /// Render todo boxes?
    with_todo: bool,
    /// Number environments?
    environment_numbers: bool,
    /// Render `noprint`?
    with_noprint: bool,
    /// Latex Sequence which separate (text) Paragraphs
    paragraph_separator: String,
    /// Space after headings
    post_heading_space: String,

    /// Templates which can be exported as an environment.
    /// The template may have a `title` attribute and a content
    /// attribute, which has the same name as the environment.
    /// Any additional template attributes will be exported as
    /// subsequent environments, if listed here.
    environments: HashMap<String, Vec<String>>,

    /// Environments which are not affected by the `environment_numbers` option.
    /// Entries always in their plain (whithout `*`) form.
    environment_numbers_exceptions: Vec<String>,
}

impl Default for LatexTarget {
    fn default() -> LatexTarget {
        LatexTarget {
            indentation_depth: 4,
            max_line_width: 80,
            image_width: 0.5,
            image_height: 0.2,
            gallery_images_per_row: 2,
            centered_image_captions: false,
            with_todo: false,
            with_noprint: true,
            environment_numbers: false,
            paragraph_separator: "".into(),
            post_heading_space: "\n".into(),
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
            environment_numbers_exceptions: string_vec!["displayquote", "figure"],
        }
    }
}

impl<'a, 's> Target<&'a LatexArgs, &'s Settings> for LatexTarget {
    fn target_type(&self) -> TargetType {
        TargetType::Latex
    }

    fn export(
        &self,
        root: &Element,
        settings: &'s Settings,
        args: &'a LatexArgs,
        out: &mut io::Write,
    ) -> io::Result<()> {
        // apply latex-specific transformations
        let mut latex_tree = root.clone();
        latex_tree = transformations::hoist_thumbnails(latex_tree, ())
            .expect("Error in thumbnail hoisting!");

        let mut renderer = LatexRenderer::new(self, &settings, &args);
        renderer.run(&latex_tree, (), out)
    }
}
