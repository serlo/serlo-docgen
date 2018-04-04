//! Definition of the LaTeX renderer. Subfunctions are implemented in other files.

use preamble::*;
use latex::LatexTarget;
use mediawiki_parser::Span;
#[macro_use]
mod blobs;

mod simple;
mod template;
mod iref;
mod list;
mod html;
mod gallery;
mod table;


/// Recursively renders a syntax tree to latex.
pub struct LatexRenderer<'e, 't> {
    pub path: Vec<&'e Element>,
    pub latex: &'t LatexTarget,
}

impl<'e, 's: 'e, 't: 'e> Traversion<'e, &'s Settings> for LatexRenderer<'e, 't> {

    path_methods!('e);

    fn work(&mut self, root: &'e Element, settings: &'s Settings,
            out: &mut io::Write) -> io::Result<bool> {

        Ok(match *root {
            // Node elements
            Element::Document { .. } => true,
            Element::Heading { .. } => self.heading(root, settings, out)?,
            Element::Formatted { .. } => self.formatted(root, settings, out)?,
            Element::Paragraph { .. } => self.paragraph(root, settings, out)?,
            Element::Template { .. } => self.template(root, settings, out)?,
            Element::TemplateArgument { .. } => self.template_arg(root, settings, out)?,
            Element::InternalReference { .. } => self.internal_ref(root, settings, out)?,
            Element::List { .. } => self.list(root, settings, out)?,
            Element::HtmlTag { .. } => self.htmltag(root, settings, out)?,
            Element::Gallery { .. } => self.gallery(root, settings, out)?,
            Element::ExternalReference { .. } => self.href(root, settings, out)?,
            Element::Table { .. } => self.table(root, settings, out)?,
            Element::TableRow { .. } => self.table_row(root, settings, out)?,


            // Leaf Elements
            Element::Text { .. } => self.text(root, settings, out)?,
            Element::Comment { .. } => self.comment(root, settings, out)?,
            Element::Error { .. } => self.error(root, settings, out)?,
            _ => {
                self.write_error(&format!("export for element `{}` not implemented!",
                    root.get_variant_name()), out)?;
                false
            },
        })
    }
}

impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {
    pub fn new(target: &LatexTarget) -> LatexRenderer {
        LatexRenderer {
            path: vec![],
            latex: target,
        }
    }

    fn environment(
        &self,
        name: &str,
        args: &[&str],
        content: &str,
        out: &mut io::Write)
    -> io::Result<()> {
        let indent = self.latex.indentation_depth;
        let line_width = self.latex.max_line_width;

        let arg_string: String = args.iter().map(|a| format!("[{}]", a)).collect();
        let content = indent_and_trim(content, indent, line_width);
        write!(out, GENERIC_ENV!(), name, &arg_string, content, name)
    }

    fn write_error(
        &self,
        message: &str,
        out: &mut io::Write)
    -> io::Result<()> {

        let message = escape_latex(message);
        self.environment("error", &[], &message, out)
    }

    fn write_def_location(&self, pos: &Span, doctitle: &str,
                          out: &mut io::Write) -> io::Result<()> {

        writeln!(out, "% defined in {} at {}:{} to {}:{}", doctitle,
                 pos.start.line, pos.start.col,
                 pos.end.line, pos.end.col)
    }

    fn error(&self,
        root: &Element,
        settings: &Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {
        if let Element::Error { ref position, ref message } = *root {
            self.write_def_location(position, &settings.document_title, out)?;
            self.write_error(message, out)?;
        }
        Ok(true)
    }
}



