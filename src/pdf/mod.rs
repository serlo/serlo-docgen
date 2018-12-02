use preamble::*;

use serde_json;
use std::io;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct PDFArgs {
    /// Title of the document beeing processed.
    document_title: String,

    /// Path to a list of link targets (anchors) available in the export.
    #[structopt(parse(from_os_str))]
    available_anchors: PathBuf,
}

/// Dump pdf settings to stdout as json.
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
            document_options: "tocflat, listof=chapterentry, parskip=half-".into(),
        }
    }
}

impl<'a, 's> Target<&'a PDFArgs, &'s Settings> for PDFTarget {
    fn target_type(&self) -> TargetType {
        TargetType::PDF
    }
    fn export(
        &self,
        _: &Element,
        settings: &'s Settings,
        args: &'a PDFArgs,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let mut data_table =
            serde_json::to_value(self).expect("could not construct value from PDFTarget!");

        let title = &settings.runtime.document_title;
        let revision = &settings.runtime.document_revision;
        if let serde_json::Value::Object(ref mut m) = data_table {
            m.insert("document_title".into(), title.clone().into());
            m.insert("document_revision".into(), revision.clone().into());
        }

        writeln!(
            out,
            "{}",
            serde_json::to_string(&data_table).expect("could not serialize the PDFTarget struct")
        )
    }
}
