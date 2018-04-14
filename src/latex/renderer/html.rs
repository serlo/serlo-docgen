use preamble::*;
use super::LatexRenderer;
use mediawiki_parser::*;


impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn htmltag(
        &mut self,
        root: &'e HtmlTag,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {

        match root.name.to_lowercase().trim() {
            "dfn" => {
                let content = root.content.render(self, settings)?;
                write!(out, HTML_ITALIC!(), &content)?;
            },
            "ref" => {
                let content = root.content.render(self, settings)?;
                write!(out, HTML_REF!(), &content)?;
            },
            "section" => (),
            _ => {
                let msg = format!("no export function defined \
                    for html tag `{}`!", root.name);
                self.write_error(&msg, out)?;
            },
        }
        Ok(false)
    }
}
