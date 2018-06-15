use preamble::*;

use std::io;
use serde_yaml;


/// Dump pdf settings to stdout as yaml.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct PDFTarget {
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
        }
    }
}

impl Target for PDFTarget {
    fn include_sections(&self) -> bool { false }
    fn target_extension(&self) -> &str { "yml" }
    fn extension_for(&self, _ext: &str) -> &str { "%" }
    fn export<'a>(&self,
                _: &'a Element,
                settings: &Settings,
                _: &[String],
                out: &mut io::Write) -> io::Result<()> {

        let mut data_table = serde_yaml::to_value(self)
            .expect("could not construct value from PDFTarget!");

        let title = &settings.runtime.document_title;
        let revision = &settings.runtime.document_revision;
        if let &mut serde_yaml::Value::Mapping(ref mut m) = &mut data_table {
            m.insert("document_title".into(), title.clone().into());
            m.insert("document_revision".into(), revision.clone().into());
        }

        writeln!(out, "{}", serde_yaml::to_string(&data_table)
            .expect("could not serialize the PDFTarget struct"))
    }
}
