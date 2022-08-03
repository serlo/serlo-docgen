//! HTMl renderer for all simple types like in the latex-renderer

use super::HtmlRenderer;
use crate::preamble::*;
use mediawiki_parser::MarkupType;

impl<'e, 's: 'e, 't: 'e, 'a> HtmlRenderer<'e, 't, 's, 'a> {
    pub fn heading(&mut self, root: &'e Heading, out: &mut dyn io::Write) -> io::Result<bool> {
        write!(
            out,
            "<h{} class=\"article-heading-{}\">",
            &root.depth, &root.depth
        )?;
        self.run_vec(&root.caption, (), out)?;
        writeln!(out, "</h{}>", &root.depth)?;
        self.run_vec(&root.content, (), out)?;
        Ok(false)
    }

    pub fn text(&mut self, root: &'e Text, out: &mut dyn io::Write) -> io::Result<bool> {
        write!(out, "{}", Self::escape_html(&root.text))?;
        Ok(false)
    }

    pub fn paragraph(&mut self, root: &'e Paragraph, out: &mut dyn io::Write) -> io::Result<bool> {
        write!(out, "<div class=\"paragraph\">")?;
        self.run_vec(&root.content, (), out)?;
        writeln!(out, "</div>")?;
        Ok(false)
    }

    pub fn comment(&mut self, root: &'e Comment, out: &mut dyn io::Write) -> io::Result<bool> {
        writeln!(out, "<!-- {} -->", Self::escape_html(&root.text))?;
        Ok(false)
    }

    pub fn href(
        &mut self,
        root: &'e ExternalReference,
        out: &mut dyn io::Write,
    ) -> io::Result<bool> {
        write!(
            out,
            "<a class=\"link\" href=\"{}\">",
            urlencode(&root.target)
        )?;
        self.run_vec(&root.caption, (), out)?;
        writeln!(out, " </a>")?;
        Ok(false)
    }

    pub fn formatted(&mut self, root: &'e Formatted, out: &mut dyn io::Write) -> io::Result<bool> {
        //let mut a = false;
        match root.markup {
            MarkupType::NoWiki => {
                write!(out, "<span class=\"nowiki\">")?;
            }
            MarkupType::Bold => {
                write!(out, "<span class=\"bold\">")?;
            }
            MarkupType::Italic => {
                write!(out, "<span class=\"italic\">")?;
            }
            MarkupType::Math => {
                self.formel(root, out)?;
                return Ok(false);
            }
            MarkupType::StrikeThrough => {
                write!(out, "<span class=\"striketrough\">")?;
            }
            MarkupType::Underline => {
                write!(out, "<span class=\"underline\">")?;
            }
            _ => {
                let msg = format!("MarkupType not implemented: {:?}", &root.markup);
                self.write_error(&msg, out)?;
            }
        }
        self.run_vec(&root.content, (), out)?;
        /*if a
        {
            write!(out, "\\)")?;
        }*/
        writeln!(out, "</span>")?;
        Ok(false)
    }

    pub fn htmltag(&mut self, root: &'e HtmlTag, out: &mut dyn io::Write) -> io::Result<bool> {
        match root.name.to_lowercase().trim() {
            "dfn" => {
                write!(out, "<dfn>")?;
                self.run_vec(&root.content, (), out)?;
                write!(out, "</dfn>")?;
            }
            // TODO: proper footnotes
            "ref" => {
                write!(out, "<sup>")?;
                self.run_vec(&root.content, (), out)?;
                write!(out, "</sup>")?;
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

    pub fn formel(&mut self, root: &'e Formatted, out: &mut dyn io::Write) -> io::Result<bool> {
        write!(out, "<span class=\"math\">")?;
        self.run_vec(&root.content, (), out)?;
        write!(out, "</span>")?;
        Ok(false)
    }
}
