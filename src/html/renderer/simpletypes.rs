
//! HTMl renderer for all simple types like in the latex-renderer

use super::HtmlRenderer;
use mediawiki_parser::MarkupType;
use preamble::*;


impl<'e, 's: 'e, 't: 'e> HtmlRenderer<'e, 't> {

    pub fn heading(
        &mut self,
        root: &'e Heading,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        write!(out, "<h{} class=\"article-heading-{}\">",&root.depth, &root.depth)?;
        self.run_vec(&root.caption,settings,out)?;
        writeln!(out, "</h{} class=\"article-heading-{}\">",&root.depth, &root.depth)?;
        self.run_vec(&root.content,settings,out)?;
        Ok(false)
    }

    pub fn text(
        &mut self,
        root: &'e Text,
        _: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        write!(out, "{}", &escape_html(&root.text))?;
        Ok(false)
    }

    pub fn paragraph(
        &mut self,
        root: &'e Paragraph,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        write!(out, "<p class=\"paragraph\">")?;
        self.run_vec(&root.content,settings,out)?;
        writeln!(out, "</p>")?;
        Ok(false)
    }

    pub fn comment(
        &mut self,
        root: &'e Comment,
        _: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        writeln!(out, "<!-- {} -->", &escape_html(&root.text))?;
        Ok(false)
    }

    pub fn href(
        &mut self,
        root: &'e ExternalReference,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        write!(out, "<a class=\"link\" href=\"{}\">", &escape_html(&root.target))?;
        self.run_vec(&root.caption,settings,out)?;
        writeln!(out, " </a>")?;
        Ok(false)
    }

    pub fn formatted(
        &mut self,
        root: &'e Formatted,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
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
                write!(out, "<span class=\"math\">")?;
                //write!(out, "\\(")?;
                //a = true;
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
        self.run_vec(&root.content,settings,out)?;
        /*if a
        {
            write!(out, "\\)")?;
        }*/
        writeln!(out, "</span>")?;
        Ok(false)
    }


    pub fn htmltag(
        &mut self,
        root: &'e HtmlTag,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        match root.name.to_lowercase().trim() {
            "dfn" | "ref" => {
                write!(out, "<{}>", &root.name.to_lowercase())?;
                self.run_vec(&root.content,settings,out)?;
                write!(out, "</{}>", &root.name.to_lowercase())?;
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
