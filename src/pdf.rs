//! Implements the `sections` target which writes out parts of the syntax tree.
//!
//! This target operates on the same syntax tree as the `deps` target. It extracts
//! parts of the document tree marked by `<section />` tags and writes them to a
//! directory specified through the transformation settings in the YAML format.

use std::collections::HashMap;
use preamble::*;

use std::io;
use serde_yaml;


/// Dump pdf settings to stdout as yaml.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct PDFTarget {
    #[serde(skip_serializing_if = "is_default")]
    pub extension_mapping: HashMap<String, String>,

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
}

impl Default for PDFTarget {
    fn default() -> PDFTarget {
        PDFTarget {
            page_trim: 0.0,
            page_width: 155.0,
            page_height: 235.0,
            font_size: 9.0,
            baseline_height: 12.0,
            border: [20.5, 32.6, 22.0, 18.5],
            document_options: "tocflat, listof=chapterentry".into(),
            extension_mapping: HashMap::new(),
        }
    }
}

impl Target for PDFTarget {
    fn do_include_sections(&self) -> bool { false }
    fn get_target_extension(&self) -> &str { "yml" }
    fn get_extension_mapping(&self) -> &HashMap<String, String> {
        &self.extension_mapping
    }
    fn export<'a>(&self,
                _: &'a Element,
                _: &Settings,
                _: &[String],
                out: &mut io::Write) -> io::Result<()> {

        writeln!(out, "{}", serde_yaml::to_string(self)
            .expect("could not serialize the PDFTarget struct"))
    }
}
