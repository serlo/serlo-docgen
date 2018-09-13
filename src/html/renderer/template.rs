use super::HtmlRenderer;
use mfnf_template_spec::*;
use mwparser_utils::*;
use preamble::*;

macro_rules! tag_wrapper {
    ($self: ident, $content:expr, $settings:ident, $out:ident, $tag:expr, $class:expr) => {
        write!($out, "<{} class=\"{}\">", $tag, $class)?;
        $self.run_vec($content, $settings, $out)?;
        write!($out, "</{}>", $tag)?;
    }
}

macro_rules! div_wrapper {
    ($self: ident, $content:expr, $settings:ident, $out:ident, $class:expr) => {
        tag_wrapper!($self, $content, $settings, $out, "div", $class)
    }
}

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
            KnownTemplate::Formula(formula) => self.formula(&formula, settings, out)?,
            KnownTemplate::Induction(induction) => self.induction(&induction, settings, out)?,
            KnownTemplate::Question(question) => self.question(&question, settings, out)?,
            KnownTemplate::ProofStep(step) => self.proofstep(&step, settings, out)?,
            KnownTemplate::ProofByCases(cases) => self.proof_by_cases(&cases, settings, out)?,
            KnownTemplate::NoPrint(noprint) => {
                self.run_vec(&noprint.content, settings, out)?;
                false
            }
            KnownTemplate::Navigation(_) => {false}
            KnownTemplate::Todo(_) => {false}
            KnownTemplate::Theorem(_)
            | KnownTemplate::Definition(_)
            | KnownTemplate::SolutionProcess(_)
            | KnownTemplate::ProofSummary(_)
            | KnownTemplate::AlternativeProof(_)
            | KnownTemplate::Proof(_)
            | KnownTemplate::Warning(_)
            | KnownTemplate::Example(_)
            | KnownTemplate::Exercise(_)
            | KnownTemplate::Hint(_) => {
                let class = parsed.identifier().to_lowercase();
                self.environment_template(&parsed, settings, out, &class)?
            }
            KnownTemplate::Solution(solution) => self.solution(&solution, settings, out)?,
            KnownTemplate::Smiley(smiley) => {
                write!(
                    out,
                    "{}",
                    smiley_to_unicode(&extract_plain_text(&smiley.name.unwrap_or(&[])))
                        .unwrap_or('\u{01f603}')
                )?;
                false
            }
            _ => {
                writeln!(out, "irgendetwas anderes")?;
                false
            }
        };
        Ok(false)
    }
    //important Todos: mainarticle: link? anchor: link?, literature, important
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
                self.run_vec(&case, settings, out)?;
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
            Element::Formatted(ref root) => match root.markup {
                MarkupType::Math => {
                    self.formel(root, settings, out)?;
                }
                _ => {
                    let msg = format!(
                        "unknown type in formula.formula[0] in formula-template! Type: {:?}",
                        root.markup
                    );
                    self.write_error(&msg, out)?;
                }
            },
            _ => {
                let msg = format!(
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
    ) -> io::Result<bool> {
        write!(out, "<details>")?;
        write!(out, "<summary class =\"question\">")?;
        if let Some(kind) = question.kind {
            write!(out, "<div class=\"question-kind\">")?;
            self.run_vec(&kind, settings, out)?;
            write!(out, ": ")?;
            write!(out, "</div>")?;
        } else {
            write!(out, "<div class=\"question-kind\">")?;
            write!(out, "Frage: ")?;
            write!(out, "</div>")?;
        }
        tag_wrapper!(self, &question.question, settings, out, "span", "question-text");
        write!(out, "</summary>")?;
        div_wrapper!(self, &question.answer, settings, out, "answer");
        write!(out, "</details>")?;
        Ok(false)
    } //it is impportant to specify in css: display: inline, otherwise weird line break

    pub fn proofstep(
        &mut self,
        step: &ProofStep<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        write!(out, "<details open>")?;
        write!(out, "<summary class =\"proofstep\">")?;
        write!(out, "Beweisschritt:")?;
        div_wrapper!(self, &step.goal, settings, out, "proofstep-goal");
        write!(out, "</summary>")?;
        div_wrapper!(self, &step.step, settings, out, "proofstep");
        write!(out, "</details>")?;
        Ok(false)
    } //auf zeilenumbrüche achten, da in einem neuen absatz reingepackt der text (seltsamerweise)

    pub fn environment_template(
        &mut self,
        template: &KnownTemplate<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
        class: &str,
    ) -> io::Result<bool> {
        //let title = template.find("title").map(|a| a.value).unwrap_or(&[]);
        //    let title_text = title.render(self, settings)?;
        write!(out, "<div class=\"{} environment\">", class)?;
        let name = match template {
            KnownTemplate::Definition(_) => "Definition",
            KnownTemplate::Theorem(_) => "Theorem",
            KnownTemplate::Example(_) => "Beispiel",
            KnownTemplate::Exercise(_) => "Aufgabe",
            KnownTemplate::Hint(_) => "Hinweis",
            KnownTemplate::Warning(_) => "Warnung",
            KnownTemplate::Proof(_) => "Beweis",
            KnownTemplate::AlternativeProof(_) => "Alternativer Beweis",
            KnownTemplate::ProofSummary(_) => "Beweiszusammenfassung",
            KnownTemplate::Solution(_) => "Lösung",
            KnownTemplate::SolutionProcess(_) => "Lösungsweg",
            _ => "FEHLER",
        };
        write!(out, "{}: ", &name)?;

        let title = template.find("title");
        if let Some(render_title) = title {
            tag_wrapper!(self, &render_title.value, settings, out, "span", "environment-title");
        }

        for attribute in template.present() {
            if attribute.name == "title" {
                continue;
            }
            let class = format!("env-{}", &attribute.name);
            div_wrapper!(self, &attribute.value, settings, out, &class);
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
        write!(out, "<details class=\"solution\"> <summary> Lösung: ")?;
        if let Some(render_title) = &solution.title {
            self.run_vec(&render_title, settings, out)?;
        }
        write!(out, "</summary>")?;
        self.run_vec(&solution.solution, settings, out)?;
        write!(out, "</details>")?;
        Ok(false)
    }
    fn induction(
        &mut self,
        induction: &Induction<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        write!(out, "<div class=\"induction\">")?;
        write!(out, "<details open><summary>")?;
        if let Some(e) = induction.basic_set {
            write!(out, "Aussageform, die wir für alle ")?;
            self.run_vec(&e, settings, out)?;
            write!(out, " beweisen wollen:")?;
        } else {
            write!(out, "Aussage die wir beweisen wollen: ")?;
        };
        write!(out, "</summary>")?;
        self.run_vec(&induction.statement, settings, out)?;
        write!(out, "</details>")?;
        //Aussage

        write!(out, "<details open><summary>")?;
        write!(out, "1. Induktionsanfang:")?;
        write!(out, "</summary>")?;
        self.run_vec(&induction.base_case, settings, out)?;
        write!(out, "</details>")?;

        write!(out, "<details open><summary>")?;
        write!(out, "2. Induktionsschritt:")?;
        write!(out, "</summary>")?;
        write!(out, "<details open><summary>")?;
        write!(out, "2.a Induktionsvoraussetzung:")?;
        write!(out, "</summary>")?;
        self.run_vec(&induction.induction_hypothesis, settings, out)?;
        write!(out, "</details>")?;
        //IV

        write!(out, "<details open><summary>")?;
        write!(out, "2.b Induktionsbehauptung:")?;
        write!(out, "</summary>")?;
        self.run_vec(&induction.step_case_goal, settings, out)?;
        write!(out, "</details>")?;
        //IB

        write!(out, "<details open><summary>")?;
        write!(out, "2.b Induktionsbehauptung:")?;
        write!(out, "</summary>")?;
        self.run_vec(&induction.step_case_goal, settings, out)?;
        write!(out, "</details>")?;

        write!(out, "<details open><summary>")?;
        write!(out, "2.c Beweis des Induktionsschritts:")?;
        write!(out, "</summary>")?;
        self.run_vec(&induction.step_case, settings, out)?;
        write!(out, "</details>")?;
        //IS

        write!(out, "</details>")?;
        write!(out, "</div>")?;

        Ok(false)
    }
}
