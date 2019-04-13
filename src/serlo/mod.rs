//! Implements the `serlo` target which transforms
//! mfnf articles to serlo articles.
use crate::preamble::*;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::io;
use structopt::StructOpt;

mod renderer;

use renderer::SerloRenderer;

#[derive(Debug, StructOpt)]
pub struct SerloArgs {
    /// Title of the document beeing processed.
    document_title: String,

    /// Path to a list of link targets (anchors) available in the export.
    #[structopt(parse(try_from_str = "load_anchor_set"))]
    available_anchors: HashSet<String>,
}

/// Transform article to serlo format.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct SerloTarget {}

impl<'a, 's> Target<&'a SerloArgs, &'s Settings> for SerloTarget {
    fn target_type(&self) -> TargetType {
        TargetType::Serlo
    }
    fn export(
        &self,
        root: &Element,
        settings: &'s Settings,
        args: &'a SerloArgs,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let mut renderer = SerloRenderer::new(self, settings, args);
        let result = renderer.run(root);

        writeln!(
            out,
            "{}",
            serde_json::to_string(&result).expect("could not serialize the rendered document")
        )
    }
}
