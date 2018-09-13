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
            //KnownTemplate::Important(important) => writeln!(out, "Important")?,//self.important(settings, &important, out)?,
            KnownTemplate::Definition(_) => self.environment_template(&parsed, settings, out, "definition")?,
            KnownTemplate::Question(question) => self.question(&question, settings, out)?,
            KnownTemplate::ProofStep(step) => self.proofstep(&step, settings, out)?,
            KnownTemplate::ProofByCases(cases) => self.proof_by_cases(&cases, settings, out)?,
            KnownTemplate::Theorem(_) => self.environment_template(&parsed, settings, out, "theorem")?,
            KnownTemplate::Example(_) => self.environment_template(&parsed, settings, out, "example")?,
            KnownTemplate::Exercise(_) => self.environment_template(&parsed, settings, out, "exercise")?,
            KnownTemplate::Hint(_) => self.environment_template(&parsed, settings, out, "hint")?,
            KnownTemplate::Warning(_) => self.environment_template(&parsed, settings, out, "warning")?,
            KnownTemplate::Proof(_) => self.environment_template(&parsed, settings, out, "proof")?,
            KnownTemplate::AlternativeProof(_) => self.environment_template(&parsed, settings, out, "alternativeproof")?,
            KnownTemplate::ProofSummary(_) => self.environment_template(&parsed, settings, out, "proofsummary")?,
            KnownTemplate::Solution(_) => self.environment_template(&parsed, settings, out, "solution")?,
            KnownTemplate::SolutionProcess(_) => self.environment_template(&parsed, settings, out, "solutionprocess")?,
            KnownTemplate::Smiley(smiley) => {
                write!(out, "{}",
                    smiley_to_unicode(&extract_plain_text(&smiley.name.unwrap_or(&[])))
                        .unwrap_or('\u{01f603}')
                )?; false
            },
            _ => {writeln!(out, "irgendetwas anderes")?; false}
        };
        Ok(false)

    }
    fn proof_by_cases(
        &mut self,
        cases: &ProofByCases<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        let attrs = [
            (Some(cases.case1), Some(cases.proof1)),
            (Some(cases.case2), Some(cases.proof2)),
            (cases.case3, cases.proof3),
            (cases.case4, cases.proof4),
            (cases.case5, cases.proof5),
            (cases.case6, cases.proof6),
        ];
        for (index, tuple) in attrs.iter().enumerate() {
            if let (Some(case), Some(proof)) = tuple {
                writeln!(out, "<div class=\"proofcase\"> Fall {}: </div>", index + 1)?;
                self.run_vec(&proof, settings, out)?;
            }
        }
        Ok(false)

    }


    fn formula(
        &mut self,
        formula: &Formula<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {

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
            return Ok(false);
        }

        writeln!(out, "<p class=\"formula\">")?;
        match formula.formula[0] {
            Element::Formatted(ref root) => {
                match root.markup {
                    MarkupType::Math => {
                        self.formel(root, settings, out)?;
                    },
                    _ => { let msg = format!(
                                        "unknown type in formula.formula[0] in formula-template! Type: {:?}",
                                        root.markup
                                    );
                        self.write_error(&msg, out)?;
                    }
                }
            },
            _ => { let msg = format!(
                                "unknown type in formula.formula[0] in formula-template! Not a Formatted: Type: {:?}",
                                formula.formula[0]
                            );
                self.write_error(&msg, out)?;
            }
        }
        writeln!(out, "</p>")?;
        Ok(false)
    }

    pub fn question(
        &mut self,
        question: &Question<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool>{
        write!(out, "<details>")?;
        write!(out, "<summary class =\"question\">")?;
        if let Some(kind) = question.kind {
            write!(out,"<div class=\"fragenart\" style=\"display: inline;\">")?;
            self.run_vec(&kind, settings, out)?;
            write!(out, ": ")?;
            write!(out,"</div>")?;
        }
        else{
            write!(out,"<div class=\"fragenart\" style=\"display: inline;\">")?;
            write!(out, "Frage: ")?;
            write!(out,"</div>")?;
        }
        self.run_vec(&question.question, settings, out)?;
        write!(out, "</summary>")?;
        write!(out,"<div class=\"answer\">")?;
        self.run_vec(&question.answer, settings, out)?;
        write!(out,"</div>")?;
        write!(out, "</details>")?;
        Ok(false)
    }//it is impportant to specify in css: display: inline, otherwise weird line break

    pub fn proofstep(
        &mut self,
        step: &ProofStep<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool>{
        write!(out, "<details open>")?;
        write!(out, "<summary class =\"proofstep\">")?;
        write!(out, "Beweisschritt: <br>")?;
        self.run_vec(&step.goal, settings, out)?;
        write!(out, "</summary>")?;
        write!(out,"<div class=\"proofstep\">")?;
        self.run_vec(&step.step, settings, out)?;
        write!(out, "</details>")?;
        Ok(false)
    }//auf zeilenumbrüche achten, da in einem neuen absatz reingepackt der text (seltsamerweise)

    pub fn environment_template(
        &mut self,
        template: &KnownTemplate<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
        typ: &str,
    ) -> io::Result<bool> {
        //let title = template.find("title").map(|a| a.value).unwrap_or(&[]);
    //    let title_text = title.render(self, settings)?;
        write!(out, "<div class=\"{}\">",typ)?;
        match template {
            KnownTemplate::Definition(_) => {
                write!(out, "Definition")?;
            },
            KnownTemplate::Theorem(_) => {
                write!(out, "Theorem")?;
            },
            KnownTemplate::Example(_) => {
                write!(out, "Beispiel")?;
            },
            KnownTemplate::Exercise(_) => {
                write!(out, "Aufgabe")?;
            },
            KnownTemplate::Hint(_) => {
                write!(out, "Hinweis")?;
            },
            KnownTemplate::Warning(_) => {
                write!(out, "Warnung")?;
            },
            KnownTemplate::Proof(_) => {
                write!(out, "Beweis")?;
            },
            KnownTemplate::AlternativeProof(_) => {
                write!(out, "Alternativer Beweis")?;
            },
            KnownTemplate::ProofSummary(_) => {
                write!(out, "Beweiszusammenfassung")?;
            },
            KnownTemplate::Solution(_) => {
                write!(out, "Lösung")?;
            },
            KnownTemplate::SolutionProcess(_) => {
                write!(out, "Lösungsweg")?;
            }
            _ => write!(out, "FEHLER")?

        }
        write!(out, ": ")?;
        let title = template.find("title");
        if let Some(render_title) = title {
            if let Element::Paragraph(ref x) = render_title.value[0] {
                self.run_vec(&x.content, settings, out)?;
            }
            else {
                self.run_vec(&render_title.value, settings, out)?;
            }
        }
        write!(out, ":")?;
        for attribute in template.present() {
            if attribute.name == "title" {
                continue;
            }
            self.run_vec(&attribute.value, settings, out)?;
        }
        write!(out, "</div>")?;

        Ok(false)
    }
    /*pub fn exercise(
        &mut self,
        exercise: &Exercise<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        write!(out, "<div class=\"aufgabe\"> Aufgabe")?;
        if let Some(title) = &exercise.title {
            write!(out, "(")?;
            self.run_vec(&title, settings, out)?;
            write!(out, ") ")?;
        };
        write(out, ":")?;
        self.run_vec(&exercise.exercise, settings, out)?;
        if let Some(solution) = &exercise.solution {
            self.run_vec(&solution, settings,out)?;
        };
        Ok(false)

    }*/
    pub fn solution(
        &mut self,
        solution: &Solution<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        write!(out, "<details class=\"solution\"> <summary> Lösung")?;
        if let Some(title) = &solution.title {
            write!(out, " (")?;
            self.run_vec(title, settings, out)?;
            write!(out, ")")?;
        }
        write!(out, " ")?;
        self.run_vec(&solution.solution, settings, out)?;
        Ok(false)
    }


}
