//! Definition of the LaTeX renderer. Subfunctions are implemented in other files.

use super::LatexTarget;
use crate::preamble::*;
#[macro_use]
mod blobs;

mod gallery;
mod html;
mod iref;
mod list;
mod simple;
mod table;
mod template;

use super::LatexArgs;

/// Recursively renders a syntax tree to latex.
pub struct LatexRenderer<'e, 't, 's: 'e, 'a> {
    pub path: Vec<&'e Element>,
    pub latex: &'t LatexTarget,

    pub settings: &'s Settings,
    pub args: &'a LatexArgs,

    /// Render paragraphs as normal text, without newline.
    pub flatten_paragraphs: bool,
}

impl<'e, 's: 'e, 't: 'e, 'a> Traversion<'e, ()> for LatexRenderer<'e, 't, 's, 'a> {
    path_methods!('e);

    fn work(&mut self, root: &'e Element, _: (), out: &mut io::Write) -> io::Result<bool> {
        Ok(match *root {
            // Node elements
            Element::Document(ref root) => self.document(root, out)?,
            Element::Heading(ref root) => self.heading(root, out)?,
            Element::Formatted(ref root) => self.formatted(root, out)?,
            Element::Paragraph(ref root) => self.paragraph(root, out)?,
            Element::Template(ref root) => self.template(root, out)?,
            Element::TemplateArgument(ref root) => self.template_arg(root, out)?,
            Element::InternalReference(ref root) => self.internal_ref(root, out)?,
            Element::List(ref root) => self.list(root, out)?,
            Element::HtmlTag(ref root) => self.htmltag(root, out)?,
            Element::Gallery(ref root) => self.gallery(root, out)?,
            Element::ExternalReference(ref root) => self.href(root, out)?,
            Element::Table(ref root) => self.table(root, out)?,
            Element::TableRow(ref root) => self.table_row(root, out)?,
            Element::TableCell(ref root) => self.table_cell(root, out)?,

            // Leaf Elements
            Element::Text(ref root) => self.text(root, out)?,
            Element::Comment(ref root) => self.comment(root, out)?,
            Element::Error(ref root) => self.error(root, out)?,
            _ => {
                self.write_error(
                    &format!(
                        "export for element `{}` not implemented!",
                        root.get_variant_name()
                    ),
                    out,
                )?;
                false
            }
        })
    }

    /// Handle paragraph line breaks correctly.
    fn work_vec(&mut self, vec: &'e [Element], _: (), out: &mut io::Write) -> io::Result<bool> {
        let mut iter = vec.iter();
        let mut current = iter.next();
        while current.is_some() {
            let next = iter.next();
            let inner = current.unwrap();

            let next_is_text = match next {
                Some(Element::Paragraph(_))
                | Some(Element::Text(_))
                | Some(Element::Formatted(_)) => true,
                _ => false,
            };
            let current_is_par = match inner {
                Element::Paragraph(_) => true,
                _ => false,
            };

            // separate paragraphs only when a paragraph (or text) follows a paragraph
            if current_is_par && next_is_text {
                let content = inner.render(self)?;
                let sep = &self.latex.paragraph_separator;
                writeln!(out, "{}{}\n", &content.trim_right(), sep)?;
            } else {
                self.run(inner, (), out)?;
            }
            current = next;
        }
        Ok(false)
    }
}

impl<'e, 's: 'e, 't: 'e, 'a> LatexRenderer<'e, 't, 's, 'a> {
    pub fn new(
        target: &'t LatexTarget,
        settings: &'s Settings,
        args: &'a LatexArgs,
    ) -> LatexRenderer<'e, 't, 's, 'a> {
        LatexRenderer {
            flatten_paragraphs: false,
            path: vec![],
            latex: target,
            settings,
            args,
        }
    }

    /// Escape LaTeX-Specific symbols
    pub fn escape_latex(input: &str) -> String {
        let mut res = String::new();
        for c in input.chars() {
            let s = match c {
                '$' => "\\$",
                '%' => "\\%",
                '&' => "\\&",
                '#' => "\\#",
                '_' => "\\_",
                '{' => "\\{",
                '}' => "\\}",
                '[' => "{[}",
                ']' => "{]}",
                '\"' => "{''}",
                '\\' => "\\textbackslash{}",
                '~' => "\\textasciitilde{}",
                '<' => "\\textless{}",
                '>' => "\\textgreater{}",
                '^' => "\\textasciicircum{}",
                '`' => "{}`", // avoid ?` and !`
                '\n' => "\\\\",
                '↯' => "\\Lightning{}",
                _ => {
                    res.push(c);
                    continue;
                }
            };
            res.push_str(s);
        }
        res
    }

    /// Render elements with flat paragraphs.
    fn run_vec_nopar(&mut self, root: &'e [Element], out: &mut io::Write) -> io::Result<()> {
        let old_par_state = self.flatten_paragraphs;
        self.flatten_paragraphs = true;
        self.run_vec(root, (), out)?;
        self.flatten_paragraphs = old_par_state;
        Ok(())
    }

    fn environment(
        &self,
        name: &str,
        args: &[&str],
        content: &str,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let indent = self.latex.indentation_depth;
        let line_width = self.latex.max_line_width;

        let arg_string: String = args.iter().map(|a| format!("[{}]", a)).collect();
        let content = indent_and_trim(content, indent, line_width);
        let is_exception = self
            .latex
            .environment_numbers_exceptions
            .contains(&name.trim_right_matches('*').trim().to_string());

        let name = if self.latex.environment_numbers || is_exception {
            name.to_string()
        } else {
            format!("{}*", name)
        };
        writeln!(out, GENERIC_ENV!(), &name, &arg_string, content, name)
    }

    fn write_error(&self, message: &str, out: &mut io::Write) -> io::Result<()> {
        let message = Self::escape_latex(message);
        self.environment("error", &[], &message, out)
    }

    fn write_def_location(
        &self,
        pos: &Span,
        doctitle: &str,
        out: &mut io::Write,
    ) -> io::Result<()> {
        writeln!(
            out,
            "% defined in {} at {}:{} to {}:{}",
            doctitle, pos.start.line, pos.start.col, pos.end.line, pos.end.col
        )
    }

    fn error(&self, root: &Error, out: &mut io::Write) -> io::Result<bool> {
        self.write_def_location(&root.position, &self.args.document_title, out)?;
        self.write_error(&root.message, out)?;
        Ok(true)
    }
}
