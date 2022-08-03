//! The `anchors` target.
//!
//! The `anchors` target extracts all valid anchors (places which can be linked to)
//! from an article. This allows to detect wether the target of a internal reference
//! is available in the export or not.

use crate::preamble::*;
use mfnf_template_spec::{parse_template, KnownTemplate};
use structopt::StructOpt;

const ANCHOR_CAPTION: &str = "Anker";

#[derive(Debug, StructOpt)]
pub struct AnchorsArgs {
    /// Title of the document beeing processed.
    doc_title: String,
}

/// Writes a list of valid anchors to the output.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AnchorsTarget {}

impl<'a> Target<&'a AnchorsArgs, ()> for AnchorsTarget {
    fn target_type(&self) -> TargetType {
        TargetType::Anchors
    }
    /// Extract dependencies from a raw source AST. Sections are
    /// not included at this point.
    fn export<'e>(
        &self,
        root: &'e Element,
        _: (),
        args: &'a AnchorsArgs,
        out: &mut dyn io::Write,
    ) -> io::Result<()> {
        let mut printer = AnchorPrinter::default();
        printer.run(root, &args.doc_title, out)?;
        writeln!(out)
    }
}

/// prints all possible link targets (anchors) within this article.
#[derive(Default)]
pub struct AnchorPrinter<'b> {
    pub path: Vec<&'b Element>,
}

impl<'a, 'b: 'a> Traversion<'a, &'b str> for AnchorPrinter<'a> {
    path_methods!('a);

    fn work(
        &mut self,
        root: &Element,
        doc_title: &'b str,
        out: &mut dyn io::Write,
    ) -> io::Result<bool> {
        if let Some(anchor) = extract_anchor(root, doc_title) {
            writeln!(out, "{}", anchor)?;
        }
        Ok(true)
    }
}

/// extract the anchor url from a template anchor
pub fn extract_template_anchor(template: &KnownTemplate, doc_title: &str) -> Option<String> {
    fn format_url(name: &str, doc_title: &str) -> String {
        format!(
            "{}#{}:{}",
            &mw_enc(doc_title),
            &mw_enc(ANCHOR_CAPTION),
            &mw_enc(name),
        )
    }
    match template {
        KnownTemplate::Anchor(ref anchor) => {
            Some(format_url(&extract_plain_text(&anchor.ref1), doc_title))
        }
        template => {
            if let Some(title) = template.find("title") {
                let text = extract_plain_text(&title.value);
                Some(format_url(&text, doc_title))
            } else {
                None
            }
        }
    }
}

/// extract the anchor url from a heading
pub fn extract_heading_anchor(heading: &Heading, doc_title: &str) -> String {
    let text = mw_enc(&extract_plain_text(&heading.caption));
    let title = mw_enc(doc_title);
    format!("{}#{}", &title, &text)
}

/// extract the anchor url from a document
pub fn extract_document_anchor(doc_title: &str) -> String {
    mw_enc(doc_title)
}

/// extract the anchor url from an element if present.
pub fn extract_anchor(root: &Element, doc_title: &str) -> Option<String> {
    match root {
        Element::Document(_) => Some(extract_document_anchor(doc_title)),
        Element::Heading(ref heading) => Some(extract_heading_anchor(heading, doc_title)),
        Element::Template(ref template) => {
            if let Some(ref template) = parse_template(template) {
                extract_template_anchor(template, doc_title)
            } else {
                None
            }
        }
        _ => None,
    }
}
