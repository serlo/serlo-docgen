//! LaTeX renderer implemenation for simple node types.

use preamble::*;
use mediawiki_parser::MarkupType;
use super::LatexRenderer;


impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn paragraph(
        &mut self, root: &'e Element,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {

        if let Element::Paragraph { ref content, .. } = *root {

            let content = content.render(self, settings)?;
            writeln!(out, "{}\n", content.trim())?;
        };
        Ok(false)
    }

    pub fn heading(
        &mut self, root: &'e Element,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {

        if let Element::Heading {depth, ref caption, ref content, .. } = *root {

            let line_width = self.latex.opts.max_line_width;
            let indent = self.latex.opts.indentation_depth;

            let caption = caption.render(self, settings)?;
            let mut content = content.render(self, settings)?;

            content = indent_and_trim(&content, indent, line_width);
            let depth_string = "sub".repeat(depth - 1);

            writeln!(out, SECTION!(), depth_string, caption.trim())?;
            writeln!(out, "{}", &content)?;
        };
        Ok(false)
    }

    pub fn comment(
        &mut self, root: &'e Element,
        _: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {

        if let Element::Comment { ref text, .. } = *root {
            writeln!(out, "% {}", &escape_latex(text))?;
        }
        Ok(false)
    }

    pub fn text(
        &mut self, root: &'e Element,
        _: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {

        if let Element::Text { ref text, .. } = *root {
            write!(out, "{}", &escape_latex(text))?;
        }
        Ok(false)
    }

    pub fn formatted(
        &mut self, root: &'e Element,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {

        if let Element::Formatted { ref markup, ref content, .. } = *root {

            let inner = content.render(self, settings)?;

            match *markup {
                MarkupType::NoWiki => {
                    write!(out, "{}", &inner)?;
                },
                MarkupType::Bold => {
                    write!(out, BOLD!(), &inner)?;
                },
                MarkupType::Italic => {
                    write!(out, ITALIC!(), &inner)?;
                },
                MarkupType::Math => {
                    let inner = extract_plain_text(content);
                    write!(out, MATH!(), &inner)?;
                },
                MarkupType::StrikeThrough => {
                    write!(out, STRIKE_THROUGH!(), &inner)?;
                },
                MarkupType::Underline => {
                    write!(out, UNDERLINE!(), &inner)?;
                },
                _ => {
                    let msg = format!("MarkupType not implemented: {:?}", &markup);
                    self.write_error(&msg, out)?;
                }
            }
        }
        Ok(false)
    }

    pub fn href(
        &mut self, root: &'e Element,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {

        if let Element::ExternalReference {
            ref target,
            ref caption,
            ..
        } = *root {
            let caption = caption.render(self, settings)?;
            writeln!(out, INTERNAL_HREF!(), target, &caption)?;
        }
        Ok(false)
    }
}
