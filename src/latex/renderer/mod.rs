//! Definition of the LaTeX renderer. Subfunctions are implemented in other files.

use preamble::*;
use latex::LatexTarget;
use mediawiki_parser::Span;

mod simple;
mod template;
mod iref;
mod list;


/// Recursively renders a syntax tree to latex.
pub struct LatexRenderer<'e, 't> {
    pub path: Vec<&'e Element>,
    pub latex: &'t LatexTarget,
}

impl<'e, 's: 'e, 't: 'e> Traversion<'e, &'s Settings> for LatexRenderer<'e, 't> {
    fn path_push(&mut self, root: &'e Element) {
        self.path.push(&root);
    }
    fn path_pop(&mut self) -> Option<&'e Element> {
        self.path.pop()
    }
    fn get_path(&self) -> &Vec<&'e Element> {
        &self.path
    }
    fn work(&mut self, root: &'e Element, settings: &'s Settings,
            out: &mut io::Write) -> io::Result<bool> {

        Ok(match root {
            // Node elements
            &Element::Document { .. } => true,
            &Element::Heading { .. } => self.heading(root, settings, out)?,
            &Element::Formatted { .. } => self.formatted(root, settings, out)?,
            &Element::Paragraph { .. } => self.paragraph(root, settings, out)?,
            &Element::Template { .. } => self.template(root, settings, out)?,
            &Element::InternalReference { .. } => self.internal_ref(root, settings, out)?,
            &Element::List { .. } => self.list(root, settings, out)?,

            // Leaf Elements
            &Element::Text { .. } => self.text(root, settings, out)?,
            &Element::Comment { .. } => self.comment(root, settings, out)?,
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

    fn write_error(&self,
                   message: &str,
                   out: &mut io::Write) -> io::Result<()> {

        let indent = self.latex.indentation_depth;
        let line_width = self.latex.max_line_width;

        let message = escape_latex(message);
        writeln!(out, "\\begin{{error}}")?;
        writeln!(out, "{}", indent_and_trim(&message, indent, line_width))?;
        writeln!(out, "\\end{{error}}")
    }

    fn write_def_location(&self, pos: &Span, doctitle: &str,
                          out: &mut io::Write) -> io::Result<()> {

        writeln!(out, "\n% defined in {} at {}:{} to {}:{}", doctitle,
                 pos.start.line, pos.start.col,
                 pos.end.line, pos.end.col)
    }
}


