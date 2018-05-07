//! Implements template rendering for latex.

use preamble::*;
use super::LatexRenderer;
use mwparser_utils::*;

impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn template(
        &mut self,
        root: &'e Template,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {

        let doctitle = &settings.document_title;
        let parsed = if let Some(parsed) = parse_template(&root) {
            parsed
        } else {
            self.write_def_location(&root.position, doctitle, out)?;
            self.write_error(&format!("template unknown or malformed: {:?}",
                &extract_plain_text(&root.name).trim().to_lowercase()
            ), out)?;
            return Ok(false)
        };

        match parsed {
            KnownTemplate::Formula(formula) => self.formula(&formula, out)?,
            KnownTemplate::Important(important) => self.important(settings, &important, out)?,
            KnownTemplate::Definition(_)
            | KnownTemplate::Theorem(_)
            | KnownTemplate::Example(_)
            | KnownTemplate::Exercise(_)
            | KnownTemplate::Hint(_)
            | KnownTemplate::Warning(_)
            | KnownTemplate::Proof(_)
            | KnownTemplate::AlternativeProof(_)
            | KnownTemplate::ProofSummary(_)
            | KnownTemplate::Solution(_)
             => self.environment_template(settings, &parsed, out)?,
            KnownTemplate::ProofStep(step) => self.proofstep(&step, settings, out)?,
            KnownTemplate::Anchor(anchor) => self.anchor(&anchor, out)?,
            KnownTemplate::Mainarticle(article) => self.mainarticle(settings, &article, out)?,
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

    fn proofstep(
        &mut self,
        step: &ProofStep<'e>,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<()> {
        let name = match step.name {
            Some(name) => name.render(self, settings)?,
            None => "<Poof Step>".into()
        };
        let goal = step.goal.render(self, settings)?;
        writeln!(out, PROOF_STEP_CAPTION!(), name.trim(), goal.trim())?;
        self.run_vec(&step.step, settings, out)
    }

    fn important(
        &mut self,
        settings: &'s Settings,
        template: &Important<'e>,
        out: &mut io::Write
    ) -> io::Result<()> {

        let content = template.content.render(self, settings)?;
        self.environment(
            IMPORTANT_ENV!(),
            &[],
            content.trim(),
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

    pub fn template_arg(
        &mut self,
        root: &'e TemplateArgument,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {
        self.run_vec(&root.value, settings, out)?;
        Ok(false)
    }

    pub fn environment_template(
        &mut self,
        settings: &'s Settings,
        template: &KnownTemplate<'e>,
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
