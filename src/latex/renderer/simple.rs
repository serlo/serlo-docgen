//! LaTeX renderer implemenation for simple node types.

use super::LatexRenderer;
use mediawiki_parser::MarkupType;
use preamble::*;
use base64;

impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {
    pub fn paragraph(
        &mut self,
        root: &'e Paragraph,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        let content = root.content.render(self, settings)?;
        if self.flatten_paragraphs {
            write!(out, "{}", content.trim())?;
        } else {
            writeln!(out, "{}\n", content.trim())?;
        }
        Ok(false)
    }

    pub fn heading(
        &mut self,
        root: &'e Heading,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        let line_width = self.latex.max_line_width;
        let indent = self.latex.indentation_depth;

        let caption = root.caption.render(self, settings)?;
        let content = root.content.render(self, settings)?;

        let content = indent_and_trim(&content, indent, line_width);
        let depth_string = "sub".repeat(root.depth - 1);

        let anchor = extract_heading_anchor(root, settings);

        writeln!(out, SECTION!(), depth_string, caption.trim())?;
        writeln!(out, LABEL!(), base64::encode(&anchor))?;
        writeln!(out, "{}", &content)?;
        Ok(false)
    }

    pub fn document(
        &mut self,
        _root: &'e Document,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        writeln!(out, LABEL!(), base64::encode(&extract_document_anchor(settings)))?;
        Ok(true)
    }

    pub fn comment(
        &mut self,
        root: &'e Comment,
        _: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        // TODO: Comments can currently cause errors with flattened paragraphs,
        // eating up following LaTeX.
        if !self.flatten_paragraphs {
            writeln!(out, "% {}", &escape_latex(&root.text).trim())?;
        }
        Ok(false)
    }

    pub fn text(
        &mut self,
        root: &'e Text,
        _: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        write!(out, "{}", &escape_latex(&root.text))?;
        Ok(false)
    }

    pub fn formatted(
        &mut self,
        root: &'e Formatted,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        let inner = root.content.render(self, settings)?;

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
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        let mut caption = root.caption.render(self, settings)?;
        if caption.is_empty() {
            caption = escape_latex(&root.target);
        }
        let url = escape_latex(&urlencode(&root.target));
        writeln!(out, INTERNAL_HREF!(), &url, &caption)?;
        Ok(false)
    }
}
