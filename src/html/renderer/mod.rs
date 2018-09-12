use html::HTMLTarget;
use preamble::*;

mod simpletypes;
mod template;
mod list;
mod media;
mod table;

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
            Element::Table(ref root) => self.table(root, settings, out)?,
            Element::TableRow(ref root) => self.table_row(root, settings, out)?,
            Element::TableCell(ref root) => self.table_cell(root, settings, out)?,
            Element::HtmlTag(ref root) => self.htmltag(root, settings, out)?,
            Element::Template(ref root) => self.template(root, settings, out)?,
            Element::List(ref root) => self.list(root, settings, out)?,
            Element::InternalReference(ref root) => self.internal_ref(root, settings, out)?,
            Element::Gallery(ref root) => self.gallery(root, settings, out)?,
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

    //error-handling
    fn write_error(&self, message: &str, out: &mut io::Write) -> io::Result<bool> {
        let message = escape_html(&(message.to_string()));
        writeln!(out, "error: {}", message)?;
        Ok(true)
    }
    /*fn error(&self, root: &Error, out: &mut io::Write) -> io::Result<bool> {
        self.write_error(&root.message, out)?;
        Ok(true)
    }*/
}


fn escape_html(stringtoreplace: &str)-> String{
        let mut x = str::replace(stringtoreplace, "<", "&lt;");
        x = str::replace(&x, ">", "&gt;");
        x
    }
