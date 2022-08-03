//! LaTeX renderer implemenation for simple node types.

use super::LatexRenderer;
use crate::anchors::{extract_document_anchor, extract_heading_anchor};
use crate::preamble::*;
use base64;
use mediawiki_parser::MarkupType;

impl<'e, 's: 'e, 't: 'e, 'a> LatexRenderer<'e, 't, 's, 'a> {
    pub fn paragraph(&mut self, root: &'e Paragraph, out: &mut dyn io::Write) -> io::Result<bool> {
        let content = root.content.render(self)?;
        if self.flatten_paragraphs {
            write!(out, "{}", content.trim())?;
        } else {
            writeln!(out, "{}", content.trim())?;
        }
        Ok(false)
    }

    pub fn heading(&mut self, root: &'e Heading, out: &mut dyn io::Write) -> io::Result<bool> {
        let line_width = self.latex.max_line_width;
        let indent = self.latex.indentation_depth;

        let caption = root.caption.render(self)?;
        let content = root.content.render(self)?;

        let content = indent_and_trim(&content, indent, line_width);
        let depth_string = "sub".repeat(root.depth - 1);

        let anchor = extract_heading_anchor(root, &self.args.document_title);

        writeln!(out, SECTION!(), depth_string, caption.trim())?;
        write!(out, "{}", " ".repeat(indent))?;
        write!(out, LABEL!(), base64::encode(&anchor))?;
        writeln!(out, "{}", &self.latex.post_heading_space)?;
        writeln!(out, "{}", &content.trim_end())?;
        Ok(false)
    }

    pub fn document(&mut self, _root: &'e Document, out: &mut dyn io::Write) -> io::Result<bool> {
        writeln!(
            out,
            LABEL!(),
            base64::encode(&extract_document_anchor(&self.args.document_title))
        )?;
        Ok(true)
    }

    pub fn comment(&mut self, root: &'e Comment, out: &mut dyn io::Write) -> io::Result<bool> {
        // TODO: Comments can currently cause errors with flattened paragraphs,
        // eating up following LaTeX.
        if !self.flatten_paragraphs {
            writeln!(out, "% {}", &Self::escape_latex(&root.text).trim())?;
        }
        Ok(false)
    }

    pub fn text(&mut self, root: &'e Text, out: &mut dyn io::Write) -> io::Result<bool> {
        write!(out, "{}", &Self::escape_latex(&root.text))?;
        Ok(false)
    }

    pub fn formatted(&mut self, root: &'e Formatted, out: &mut dyn io::Write) -> io::Result<bool> {
        let inner = root.content.render(self)?;

        match root.markup {
            MarkupType::NoWiki => {
                write!(out, "{}", &inner)?;
            }
            MarkupType::Bold => {
                write!(out, BOLD!(), &inner)?;
            }
            MarkupType::Italic => {
                write!(out, ITALIC!(), &inner)?;
            }
            MarkupType::Math => {
                let inner = extract_plain_text(&root.content);
                write!(out, MATH!(), &inner)?;
            }
            MarkupType::StrikeThrough => {
                write!(out, STRIKE_THROUGH!(), &inner)?;
            }
            MarkupType::Underline => {
                write!(out, UNDERLINE!(), &inner)?;
            }
            MarkupType::Blockquote => {
                self.environment(QUOTE_ENV!(), &[], &inner, out)?;
            }
            _ => {
                let msg = format!("MarkupType not implemented: {:?}", &root.markup);
                self.write_error(&msg, out)?;
            }
        }
        Ok(false)
    }

    pub fn href(
        &mut self,
        root: &'e ExternalReference,
        out: &mut dyn io::Write,
    ) -> io::Result<bool> {
        let mut caption = root.caption.render(self)?;
        if caption.is_empty() {
            caption = Self::escape_latex(&root.target);
        }
        let url = Self::escape_latex(&urlencode(&root.target));
        writeln!(out, INTERNAL_HREF!(), &url, &caption)?;
        Ok(false)
    }
}
