use super::LatexRenderer;
use crate::preamble::*;
use mediawiki_parser::*;

impl<'e, 's: 'e, 't: 'e, 'a> LatexRenderer<'e, 't, 's, 'a> {
    pub fn htmltag(&mut self, root: &'e HtmlTag, out: &mut io::Write) -> io::Result<bool> {
        match root.name.to_lowercase().trim() {
            "dfn" => {
                let content = root.content.render(self)?;
                write!(out, HTML_ITALIC!(), &content)?;
            }
            "ref" => {
                let content = root.content.render(self)?;
                write!(out, HTML_REF!(), &content)?;
            }
            "section" => (),
            _ => {
                let msg = format!(
                    "no export function defined \
                     for html tag `{}`!",
                    root.name
                );
                self.write_error(&msg, out)?;
            }
        }
        Ok(false)
    }
}
