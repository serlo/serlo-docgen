use super::{HTMLArgs, HTMLTarget};
use crate::preamble::*;

mod list;
mod media;
mod simpletypes;
mod table;
mod template;

pub struct HtmlRenderer<'e, 't, 's, 'a> {
    pub path: Vec<&'e Element>,
    pub html: &'t HTMLTarget,

    pub settings: &'s Settings,
    pub args: &'a HTMLArgs,
}

impl<'e, 's: 'e, 't: 'e, 'a> Traversion<'e, ()> for HtmlRenderer<'e, 't, 's, 'a> {
    path_methods!('e);

    fn work(&mut self, root: &'e Element, _: (), out: &mut dyn io::Write) -> io::Result<bool> {
        //writeln!(out, "{}", root.get_variant_name())?;
        Ok(match *root {
            // Node elements
            Element::Document(_) => true,
            Element::Heading(ref root) => self.heading(root, out)?,
            Element::Text(ref root) => self.text(root, out)?,
            Element::Paragraph(ref root) => self.paragraph(root, out)?,
            Element::Comment(ref root) => self.comment(root, out)?,
            Element::ExternalReference(ref root) => self.href(root, out)?,
            Element::Formatted(ref root) => self.formatted(root, out)?,
            Element::Table(ref root) => self.table(root, out)?,
            Element::TableRow(ref root) => self.table_row(root, out)?,
            Element::TableCell(ref root) => self.table_cell(root, out)?,
            Element::HtmlTag(ref root) => self.htmltag(root, out)?,
            Element::Template(ref root) => self.template(root, out)?,
            Element::List(ref root) => self.list(root, out)?,
            Element::InternalReference(ref root) => self.internal_ref(root, out)?,
            Element::Gallery(ref root) => self.gallery(root, out)?,
            _ => {
                writeln!(out, "all other types")?;
                true
            }
        })
    }
}
impl<'e, 's: 'e, 't: 'e, 'a> HtmlRenderer<'e, 't, 's, 'a> {
    pub fn new(
        target: &'t HTMLTarget,
        settings: &'s Settings,
        args: &'a HTMLArgs,
    ) -> HtmlRenderer<'e, 't, 's, 'a> {
        HtmlRenderer {
            path: vec![],
            html: target,
            settings,
            args,
        }
    }

    pub fn escape_html(input: &str) -> String {
        let mut res = String::new();
        for c in input.chars() {
            let s = match c {
                '<' => "&lt;",
                '>' => "&gt;",
                '&' => "&amp;",
                '"' => "&quot;",
                '\'' => "&#39;",
                _ => {
                    res.push(c);
                    continue;
                }
            };
            res.push_str(s);
        }
        res
    }

    //error-handling
    fn write_error(&self, message: &str, out: &mut dyn io::Write) -> io::Result<bool> {
        let message = Self::escape_html(&(message.to_string()));
        writeln!(out, "error: {}", message)?;
        Ok(true)
    }
    fn error(&self, root: &Error, out: &mut dyn io::Write) -> io::Result<bool> {
        self.write_error(&root.message, out)?;
        Ok(true)
    }
}
