//! LaTeX renderer implemenation for simple node types.

use preamble::*;
use mediawiki_parser::MarkupType;
use super::LatexRenderer;


impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn paragraph(&mut self, root: &'e Element,
                     settings: &'s Settings,
                     out: &mut io::Write) -> io::Result<bool> {

        if let Element::Paragraph { ref content, .. } = *root {

            let content = content.render(self, settings)?;
            writeln!(out, "{}\n", content.trim())?;
        };
        Ok(false)
    }

    pub fn heading(&mut self, root: &'e Element,
                   settings: &'s Settings,
                   out: &mut io::Write) -> io::Result<bool> {

        if let Element::Heading {depth, ref caption, ref content, .. } = *root {

            let line_width = self.latex.max_line_width;
            let indent = self.latex.indentation_depth;

            let caption = caption.render(self, settings)?;
            let mut content = content.render(self, settings)?;

            content = indent_and_trim(&content, indent, line_width);
            let depth_string = "sub".repeat(depth - 1);

            writeln!(out, SECTION!(), depth_string, caption.trim())?;
            writeln!(out, "{}", &content);
        };
        Ok(false)
    }

    pub fn comment(&mut self, root: &'e Element,
                   _: &'s Settings,
                   out: &mut io::Write) -> io::Result<bool> {

        if let Element::Comment { ref text, .. } = *root {
            writeln!(out, "% {}", text)?;
        }
        Ok(false)
    }

    pub fn text(&mut self, root: &'e Element,
                _: &'s Settings,
                out: &mut io::Write) -> io::Result<bool> {

        if let Element::Text { ref text, .. } = *root {
            write!(out, "{}", &escape_latex(text))?;
        }
        Ok(false)
    }

    pub fn formatted(&mut self, root: &'e Element,
                     settings: &'s Settings,
                     out: &mut io::Write) -> io::Result<bool> {

        if let Element::Formatted { ref markup, ref content, .. } = *root {
            match *markup {
                MarkupType::NoWiki => {
                    self.run_vec(content, settings, out)?;
                },
                MarkupType::Bold => {
                    write!(out, "\\textbf{{")?;
                    self.run_vec(content, settings, out)?;
                    write!(out, "}}")?;
                },
                MarkupType::Italic => {
                    write!(out, "\\textit{{")?;
                    self.run_vec(content, settings, out)?;
                    write!(out, "}}")?;

                },
                MarkupType::Math => {
                    write!(out, "${}$", match content.first() {
                        Some(&Element::Text {ref text, .. }) => text,
                        _ => "parse error!",
                    })?;
                },
                MarkupType::StrikeThrough => {
                    write!(out, "\\sout{{")?;
                    self.run_vec(content, settings, out)?;
                    write!(out, "}}")?;
                },
                MarkupType::Underline => {
                    writeln!(out, "\\ul{{")?;
                    self.run_vec(content, settings, out)?;
                    writeln!(out, "}}")?;
                },
                _ => {
                    let msg = format!("MarkupType not implemented: {:?}", &markup);
                    self.write_error(&msg, out)?;
                }
            }
        }
        Ok(false)
    }
}
