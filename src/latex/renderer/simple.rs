//! LaTeX renderer implemenation for simple node types.

use preamble::*;
use mediawiki_parser::MarkupType;
use super::LatexRenderer;


impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn paragraph(&mut self, root: &'e Element,
                     settings: &'s Settings,
                     out: &mut io::Write) -> io::Result<bool> {

        if let &Element::Paragraph { ref content, .. } = root {

            // render paragraph content
            let mut par_content = vec![];
            self.run_vec(content, settings, &mut par_content)?;
            let par_string = String::from_utf8(par_content)
                .unwrap().trim_right().to_string();

            let indent = self.latex.indentation_depth;
            let line_width = self.latex.max_line_width;

            // trim and indent output string
            let trimmed = indent_and_trim(&par_string, indent, line_width);
            writeln!(out, "{}\n", &trimmed)?;
        };
        Ok(false)
    }

    pub fn heading(&mut self, root: &'e Element,
                   settings: &'s Settings,
                   out: &mut io::Write) -> io::Result<bool> {

        if let &Element::Heading {ref depth, ref caption, ref content, .. } = root {

            write!(out, "\\")?;

            for _ in 1..*depth {
                write!(out, "sub")?;
            }

            write!(out, "section{{")?;
            self.run_vec(caption, settings, out)?;
            write!(out, "}}\n\n")?;

            self.run_vec(content, settings, out)?;
        };
        Ok(false)
    }

    pub fn comment(&mut self, root: &'e Element,
                   _: &'s Settings,
                   out: &mut io::Write) -> io::Result<bool> {

        if let &Element::Comment { ref text, .. } = root {
            writeln!(out, "% {}", text)?;
        }
        Ok(false)
    }

    pub fn text(&mut self, root: &'e Element,
                _: &'s Settings,
                out: &mut io::Write) -> io::Result<bool> {

        if let &Element::Text { ref text, .. } = root {
            write!(out, "{}", &escape_latex(text))?;
        }
        Ok(false)
    }

    pub fn formatted(&mut self, root: &'e Element,
                     settings: &'s Settings,
                     out: &mut io::Write) -> io::Result<bool> {

        if let &Element::Formatted { ref markup, ref content, .. } = root {
            match markup {
                &MarkupType::NoWiki => {
                    self.run_vec(content, settings, out)?;
                },
                &MarkupType::Bold => {
                    write!(out, "\\textbf{{")?;
                    self.run_vec(content, settings, out)?;
                    write!(out, "}}")?;
                },
                &MarkupType::Italic => {
                    write!(out, "\\textit{{")?;
                    self.run_vec(content, settings, out)?;
                    write!(out, "}}")?;

                },
                &MarkupType::Math => {
                    write!(out, "${}$", match content.first() {
                        Some(&Element::Text {ref text, .. }) => text,
                        _ => "parse error!",
                    })?;
                },
                &MarkupType::StrikeThrough => {
                    write!(out, "\\sout{{")?;
                    self.run_vec(content, settings, out)?;
                    write!(out, "}}")?;
                },
                &MarkupType::Underline => {
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
