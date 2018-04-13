//! Implements template rendering for latex.

use preamble::*;
use super::LatexRenderer;
use mfnf_commons::*;

impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn template(&mut self, root: &'e Element,
                       settings: &'s Settings,
                       out: &mut io::Write) -> io::Result<bool> {

        let doctitle = &settings.document_title;
        let parsed = if let Some(parsed) = parse_template(root) {
            parsed
        } else {
            self.write_def_location(root.get_position(), doctitle, out)?;
            self.write_error(&format!("template unknown or malformed: {:?}",
                &(if let Element::Template { ref name, .. } = *root {
                    extract_plain_text(name).trim().to_lowercase()
                } else { "not a template".into() })
            ), out)?;
            return Ok(false)
        };

        match parsed {
            Template::Formula(formula) => self.formula(&formula, out)?,
            Template::Important(important) => self.important(&important, out)?,
            Template::Definition(_)
            | Template::Theorem(_)
            | Template::Example(_)
             => self.environment_template(settings, &parsed, out)?,
            Template::Anchor(anchor) => self.anchor(&anchor, out)?,
            Template::Mainarticle(article) => self.mainarticle(settings, &article, out)?,
        };
        Ok(false)
    }

    fn formula(&self, formula: &Formula, out: &mut io::Write) -> io::Result<()> {
        let content = extract_plain_text(formula.formula).trim().to_owned();

        let mut trimmed = trim_enclosing(
            &content,
            "\\begin{align}",
            "\\end{align}"
        );
        trimmed = trim_enclosing(
            trimmed,
            "\\begin{align*}",
            "\\end{align*}"
        ).trim();

        self.environment(
            MATH_ENV!(),
            &[],
            trimmed,
            out,
        )
    }

    fn important(&self, template: &Important, out: &mut io::Write) -> io::Result<()> {

        self.environment(
            IMPORTANT_ENV!(),
            &[],
            extract_plain_text(template.content).trim(),
            out
        )
    }

    fn anchor(&self, template: &Anchor, out: &mut io::Write) -> io::Result<()> {
        for reference in &template.present {
            write!(out, LABEL!(), extract_plain_text(&reference.value).trim())?;
        }
        Ok(())
    }

    fn mainarticle(
        &mut self,
        settings: &'s Settings,
        template: &Mainarticle<'e>,
        out: &mut io::Write
     ) -> io::Result<()> {

        let name = extract_plain_text(template.article)
            .trim().to_owned();
        let mut url = settings.article_url_base.to_owned();
        url.push_str(&name);
        url = url.replace(' ', "_");

        write!(out, MAINARTICLE!(), &url, &name)
    }

    pub fn template_arg(&mut self, root: &'e Element,
                    settings: &'s Settings,
                    out: &mut io::Write) -> io::Result<bool> {

        if let Element::TemplateArgument {
            ref value,
            ..
        } = *root {
            self.run_vec(value, settings, out)?;
        }
        Ok(false)
    }

    pub fn environment_template(
        &mut self,
        settings: &'s Settings,
        template: &Template<'e>,
        out: &mut io::Write
    ) -> io::Result<()> {
        let title = template.find("title").map(|a| a.value).unwrap_or(&[]);
        let title_text = title.render(self, settings)?;
        for attribute in template.present() {
            if attribute.name == "title".to_string() {
                continue
            }

            let content = attribute.value.render(self, settings)?;
            self.environment(
                &attribute.name,
                &[title_text.trim()],
                content.trim(),
                out
            )?;
        }
        Ok(())
    }
}
