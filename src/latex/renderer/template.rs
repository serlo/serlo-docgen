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

        match *parsed.id() {
            TemplateID::Formula => {
                self.formula(&parsed, out)?;
            },
            TemplateID::Important => {
                self.important(&parsed, out)?;
            },
            TemplateID::Definition
            | TemplateID::Theorem
            | TemplateID::Example
             => {
                self.environment_template(settings, &parsed, out)?;
            }
            TemplateID::Anchor => {
                self.anchor(&parsed, out)?;
            }
        };
        Ok(false)
    }

    fn formula(&self, template: &Template, out: &mut io::Write) -> io::Result<()> {

        if let Template::Formula { ref formel, .. } = *template {
            let content = extract_plain_text(formel.unwrap_or(&[])).trim().to_owned();

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

            return self.environment(
                MATH_ENV!(),
                &[],
                trimmed,
                out,
            );
        }
        unreachable!();
    }

    fn important(&self, template: &Template, out: &mut io::Write) -> io::Result<()> {

        if let Template::Important { ref content, .. } = *template {
            return self.environment(
                IMPORTANT_ENV!(),
                &[],
                extract_plain_text(content.unwrap_or(&[])).trim(),
                out
            );
        }
        unreachable!();
    }

    fn anchor(&self, template: &Template, out: &mut io::Write) -> io::Result<()> {
        for reference in template.present() {
            write!(out, LABEL!(), extract_plain_text(&reference.value).trim())?;
        }
        Ok(())
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
    ) -> io::Result<bool> {
        let title = if let Some(attr) = template.find("title") {
            attr.value
        } else {
            &[]
        };
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
        Ok(true)
    }
}
