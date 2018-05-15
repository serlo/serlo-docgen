//! Definition of the LaTeX renderer. Subfunctions are implemented in other files.

use preamble::*;
use latex::LatexTarget;
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
    /// Render paragraphs as normal text, without newline.
    pub flatten_paragraphs: bool,
}

impl<'e, 's: 'e, 't: 'e> Traversion<'e, &'s Settings> for LatexRenderer<'e, 't> {

    path_methods!('e);

    fn work(&mut self, root: &'e Element, settings: &'s Settings,
            out: &mut io::Write) -> io::Result<bool> {

        Ok(match *root {
            // Node elements
            Element::Document(_) => true,
            Element::Heading(ref root) => self.heading(root, settings, out)?,
            Element::Formatted(ref root) => self.formatted(root, settings, out)?,
            Element::Paragraph(ref root) => self.paragraph(root, settings, out)?,
            Element::Template(ref root) => self.template(root, settings, out)?,
            Element::TemplateArgument(ref root) => self.template_arg(root, settings, out)?,
            Element::InternalReference(ref root) => self.internal_ref(root, settings, out)?,
            Element::List(ref root) => self.list(root, settings, out)?,
            Element::HtmlTag(ref root) => self.htmltag(root, settings, out)?,
            Element::Gallery(ref root) => self.gallery(root, settings, out)?,
            Element::ExternalReference(ref root) => self.href(root, settings, out)?,
            Element::Table(ref root) => self.table(root, settings, out)?,
            Element::TableRow(ref root) => self.table_row(root, settings, out)?,
            Element::TableCell(ref root) => self.table_cell(root, settings, out)?,


            // Leaf Elements
            Element::Text(ref root) => self.text(root, settings, out)?,
            Element::Comment(ref root) => self.comment(root, settings, out)?,
            Element::Error(ref root) => self.error(root, settings, out)?,
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
            flatten_paragraphs: false,
            path: vec![],
            latex: target,
        }
    }

    /// Render an element with flat paragraphs.
    fn run_nopar(
        &mut self,
        root: &'e Element,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<()> {
        let old_par_state = self.flatten_paragraphs;
        self.flatten_paragraphs = true;
        self.run(root, settings, out)?;
        self.flatten_paragraphs = old_par_state;
        Ok(())
    }

    /// Render elements with flat paragraphs.
    fn run_vec_nopar(
        &mut self,
        root: &'e [Element],
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<()> {
        let old_par_state = self.flatten_paragraphs;
        self.flatten_paragraphs = true;
        self.run_vec(root, settings, out)?;
        self.flatten_paragraphs = old_par_state;
        Ok(())
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
        root: &Error,
        settings: &Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {
        self.write_def_location(&root.position, &settings.runtime.document_title, out)?;
        self.write_error(&root.message, out)?;
        Ok(true)
    }
}



