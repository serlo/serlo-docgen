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
            KnownTemplate::Formula(formula) => self.formula(&formula, settings, out)?,//self.formula(&formula, settings, out)?,
            KnownTemplate::Important(important) => writeln!(out, "Important")?,//self.important(settings, &important, out)?,
            KnownTemplate::Definition(_) => self.environment_template(root, settings, out, "definition")?,
            KnownTemplate::Theorem(_) => self.environment_template(root, settings, out, "theorem")?,
            KnownTemplate::Example(_) => self.environment_template(root, settings, out, "example")?,
            KnownTemplate::Exercise(_) => self.environment_template(root, settings, out, "exercise")?,
            KnownTemplate::Hint(_) => self.environment_template(root, settings, out, "hint")?,
            KnownTemplate::Warning(_) => self.environment_template(root, settings, out, "warning")?,
            KnownTemplate::Proof(_) => self.environment_template(root, settings, out, "proof")?,
            KnownTemplate::AlternativeProof(_) => self.environment_template(root, settings, out, "alternativeproof")?,
            KnownTemplate::ProofSummary(_) => self.environment_template(root, settings, out, "proofsummary")?,
            KnownTemplate::Solution(_) => self.environment_template(root, settings, out, "solution")?,
            KnownTemplate::SolutionProcess(_) => self.environment_template(root, settings, out, "solutionprocess")?,
            KnownTemplate::Smiley(smiley) => write!(
                out,
                "{}",
                smiley_to_unicode(&extract_plain_text(&smiley.name.unwrap_or(&[])))
                    .unwrap_or('\u{01f603}')
            )?,
            _ => writeln!(out, "irgendetwas anderes")?
        };
        Ok(false)

    }

    fn formula(
        &mut self,
        formula: &Formula<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<()> {

        let error = formula
            .formula
            .iter()
            .filter_map(|e| {
                if let Element::Error(ref err) = e {
                    Some(err)
                } else {
                    None
                }
            })
            .next();

        if let Some(err) = error {
            self.error(err, out)?;
            return Ok(());
        }

        writeln!(out, "<p class=\"formula\">")?;
        match formula.formula[0] {
            Element::Formatted(ref root) => {
                match root.markup {
                    MarkupType::Math => {
                        self.formel(root, settings, out).map(|_| ())?
                    },
                    _ => write!(out, "FEHLER")?
                }
            },
            _ => write!(out, "FEHLER")?
        }
        writeln!(out, "</p>")?;
        Ok(())
    }

    pub fn environment_template(
        &mut self,
        root: &'e Template,
        settings: &'s Settings,
        out: &mut io::Write,
        typ: &str,
    ) -> io::Result<()> {
        //let title = template.find("title").map(|a| a.value).unwrap_or(&[]);
    //    let title_text = title.render(self, settings)?;
        write!(out, "<div class=\"{}\"",typ)?;
        self.run_vec(&root.content, settings, out)?;
        write!(out, "</div>")?;
        Ok(())
    }


}
