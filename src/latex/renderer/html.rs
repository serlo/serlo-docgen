use preamble::*;
use super::LatexRenderer;
use mediawiki_parser::*;


impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn htmltag(&mut self, root: &'e Element,
                   settings: &'s Settings,
                   out: &mut io::Write) -> io::Result<bool> {
        if let Element::HtmlTag {
            ref position,
            ref name,
            ref attributes,
            ref content
        } = *root {
            match name.to_lowercase().trim() {
                "dfn" => {
                    let content = content.render(self, settings)?;
                    write!(out, HTML_ITALIC!(), &content)?;
                },
                "ref" => {
                    let content = content.render(self, settings)?;
                    write!(out, HTML_REF!(), &content)?;
                },
                _ => {
                    let msg = format!("no export function defined \
                        for html tag `{}`!", name);
                    self.write_error(&msg, out)?;
                },
            }
        }
        Ok(false)
    }
}
