use super::HtmlRenderer;
use mfnf_template_spec::*;
use mwparser_utils::*;
use preamble::*;


impl<'e, 's: 'e, 't: 'e> HtmlRenderer<'e, 't> {
    pub fn template(
        &mut self,
        root: &'e Template,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        let doctitle = &settings.runtime.document_title;
        let parsed = if let Some(parsed) = parse_template(&root) {
            parsed
        } else {
            //self.write_def_location(&root.position, doctitle, out)?;
            self.write_error(
                &format!(
                    "template unknown or malformed: {:?}",
                    &extract_plain_text(&root.name).trim().to_lowercase()
                ),
                out,
            )?;
            return Ok(false);
        };

        match parsed {
            KnownTemplate::Formula(formula) => writeln!(out, "formula")?,//self.formula(&formula, settings, out)?,
            KnownTemplate::Important(important) => writeln!(out, "Important")?,//self.important(settings, &important, out)?,
            KnownTemplate::Definition(_) => self.environment_template(root, settings, &parsed, out, "definition")?,
            KnownTemplate::Theorem(_) => self.environment_template(root, settings, &parsed, out, "theorem")?,
            KnownTemplate::Example(_) => self.environment_template(root, settings, &parsed, out, "example")?,
            KnownTemplate::Exercise(_) => self.environment_template(root, settings, &parsed, out, "exercise")?,
            KnownTemplate::Hint(_) => self.environment_template(root, settings, &parsed, out, "hint")?,
            KnownTemplate::Warning(_) => self.environment_template(root, settings, &parsed, out, "warning")?,
            KnownTemplate::Proof(_) => self.environment_template(root, settings, &parsed, out, "proof")?,
            KnownTemplate::AlternativeProof(_) => self.environment_template(root, settings, &parsed, out, "alternativeproof")?,
            KnownTemplate::ProofSummary(_) => self.environment_template(root, settings, &parsed, out, "proofsummary")?,
            KnownTemplate::Solution(_) => self.environment_template(root, settings, &parsed, out, "solution")?,
            KnownTemplate::SolutionProcess(_) => self.environment_template(root, settings, &parsed, out, "solutionprocess")?,
            _ => writeln!(out, "irgendetwas anderes")?
        };
        Ok(false)

    }

    pub fn environment_template(
        &mut self,
        root: &'e Template,
        settings: &'s Settings,
        template: &KnownTemplate<'e>,
        out: &mut io::Write,
        typ: &str,
    ) -> io::Result<()> {
        write!(out, "<div class=\"{}\"",typ)?;
        self.run_vec(&root.content, settings, out);
        write!(out, "</div>")?;
        Ok(())
    }



}
