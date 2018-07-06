use html::HTMLTarget;
use preamble::*;

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
            Element::Formatted(ref root) => {
                writeln!(out, "2")?;
                true
            },
            Element::Paragraph(ref root) =>{
                writeln!(out, "3")?;
                true
            },
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
        write!(out, "{}", &root.text)?;
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
        writeln!(out, "<!-- {} -->", &root.text)?;
        Ok(false)
    }

    pub fn href(
        &mut self,
        root: &'e ExternalReference,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        write!(out, "<a class=\"link\" href=\"{}\">", &root.target)?;
        self.run_vec(&root.caption,settings,out)?;
        writeln!(out, " </a>")?;
        Ok(false)
    }

/*    pub fn formatted(
        &mut self,
        root: &'e Formatted,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {

        write!(out, "")
        let inner = root.content.render(self, settings)?;

        match root.markup {
            MarkupType::NoWiki => {
                write!(out, "{}", &inner)?;
            }
            MarkupType::Bold => {
                write!(out, BOLD!(), &inner)?;
            }
            MarkupType::Italic => {
                write!(out, ITALIC!(), &inner)?;
            }
            MarkupType::Math => {
                let inner = extract_plain_text(&root.content);
                write!(out, MATH!(), &inner)?;
            }
            MarkupType::StrikeThrough => {
                write!(out, STRIKE_THROUGH!(), &inner)?;
            }
            MarkupType::Underline => {
                write!(out, UNDERLINE!(), &inner)?;
            }
            _ => {
                let msg = format!("MarkupType not implemented: {:?}", &root.markup);
                self.write_error(&msg, out)?;
            }
        }
        Ok(false)
    }
*/
/*
    pub fn htmltag(
        &mut self,
        root: &'e HtmlTag,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        match root.name.to_lowercase().trim() {
            "dfn" => {
                self.run_vec(&root.content,settings,out)?;
            }
            "ref" => {
                let content = root.content.render(self, settings)?;
                write!(out, "{}", &content)?;
            }
            "section" => (),
            _ => {
                let msg = format!(
                    "no export function defined \
                     for html tag `{}`!",
                    root.name
                );

            }
        }
        Ok(false)
    }*/
}
