use html::HTMLTarget;
use preamble::*;

mod simpletypes;

pub struct HtmlRenderer<'e, 't> {
    pub path: Vec<&'e Element>,
    pub html: &'t HTMLTarget,
}

impl<'e, 's: 'e, 't: 'e> Traversion<'e, &'s Settings> for HtmlRenderer<'e, 't> {
    path_methods!('e);

    fn work(
        &mut self,
        root: &'e Element,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        //writeln!(out, "{}", root.get_variant_name())?;
        Ok(match *root {
            // Node elements
            Element::Document(_) => true,
            Element::Heading(ref root) => self.heading(root, settings, out)?,
            Element::Text(ref root) => self.text(root, settings, out)?,
            Element::Paragraph(ref root) => self.paragraph(root, settings, out)?,
            Element::Comment(ref root) => self.comment(root, settings, out)?,
            Element::ExternalReference(ref root) => self.href(root, settings, out)?,
            Element::Formatted(ref root) => self.formatted(root, settings, out)?,
            Element::Template(ref root) => {
                writeln!(out, "4")?;
                true
            },
            _ => {
                writeln!(out, "all other types")?;
                true
            }
        })
    }
}
impl<'e, 's: 'e, 't: 'e> HtmlRenderer<'e, 't> {
    pub fn new(target: &HTMLTarget) -> HtmlRenderer {
        HtmlRenderer {
            path: vec![],
            html: target,
        }
    }
}
