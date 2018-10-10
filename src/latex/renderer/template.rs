//! Implements template rendering for latex.

use super::LatexRenderer;
use base64;
use mfnf_template_spec::*;
use mwparser_utils::*;
use preamble::*;

impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {
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
            self.write_def_location(&root.position, doctitle, out)?;
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
            KnownTemplate::Important(important) => self.important(settings, &important, out)?,
            KnownTemplate::Literature(literature) => self.literature(&literature, out)?,
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
            | KnownTemplate::SolutionProcess(_) => {
                self.environment_template(settings, &parsed, out)?
            }
            KnownTemplate::GroupExercise(group) => self.group_exercise(&group, settings, out)?,
            KnownTemplate::ProofStep(step) => self.proofstep(&step, settings, out)?,
            KnownTemplate::Anchor(_) => self.anchor(&parsed, settings, out)?,
            KnownTemplate::Mainarticle(article) => self.mainarticle(settings, &article, out)?,
            KnownTemplate::Navigation(_) => (),
            KnownTemplate::Question(question) => self.question(&question, settings, out)?,
            KnownTemplate::ProofByCases(cases) => self.proof_by_cases(&cases, settings, out)?,
            KnownTemplate::Induction(induction) => self.induction(&induction, settings, out)?,
            KnownTemplate::Smiley(smiley) => write!(
                out,
                "{}",
                smiley_to_unicode(&extract_plain_text(&smiley.name.unwrap_or(&[])))
                    .unwrap_or('\u{01f603}')
            )?,
            // TODO: replace noprint with a sematic version, ignore for now.
            KnownTemplate::NoPrint(noprint) => self.run_vec(&noprint.content, settings, out)?,
            KnownTemplate::Todo(todo) => self.todo(&todo, settings, out)?,
        };
        Ok(false)
    }

    fn formula(
        &self,
        formula: &Formula,
        settings: &Settings,
        out: &mut io::Write,
    ) -> io::Result<()> {
        // propagate errors
        let error = formula
            .formula
            .iter()
            .filter_map(|e| {
                if let Element::Error(ref err) = e {
                    Some(err)
                } else {
                    None
                }
            }).next();

        if let Some(err) = error {
            self.error(err, settings, out)?;
            return Ok(());
        }

        let content = extract_plain_text(formula.formula).trim().to_owned();

        let mut trimmed = trim_enclosing(&content, "\\begin{align}", "\\end{align}");
        trimmed = trim_enclosing(trimmed, "\\begin{align*}", "\\end{align*}").trim();

        // align environments need to be separated from surrounding text
        writeln!(out, "\n")?;
        self.environment(MATH_ENV!(), &[], trimmed, out)
    }

    fn proofstep(
        &mut self,
        step: &ProofStep<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let name = match step.name {
            Some(name) => name.render(self, settings)?,
            None => "Beweisschritt".into(),
        };
        let goal = step.goal.render(self, settings)?;
        writeln!(out, PROOF_STEP_CAPTION!(), name.trim(), goal.trim())?;
        self.run_vec(&step.step, settings, out)
    }

    fn todo(
        &mut self,
        todo: &Todo<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<()> {
        if self.latex.with_todo {
            let text = todo.todo.render(self, settings)?;
            self.environment("todo", &[], &format!("{}\n", text.trim()), out)?;
        }
        Ok(())
    }

    fn literature(&mut self, literature: &Literature<'e>, out: &mut io::Write) -> io::Result<()> {
        let mut lit = String::new();
        if let Some(author) = literature.author {
            lit.push_str(&extract_plain_text(author));
            lit.push_str(": ");
        }
        lit.push_str(&format!(
            "\\emph{{{}}}",
            &extract_plain_text(literature.title)
        ));

        if let Some(publisher) = literature.publisher {
            lit.push_str(". ");
            lit.push_str(&extract_plain_text(publisher));
        }
        if let Some(address) = literature.address {
            lit.push_str(", ");
            lit.push_str(&extract_plain_text(address));
        }
        if let Some(year) = literature.year {
            lit.push_str(" ");
            lit.push_str(&extract_plain_text(year));
        }
        if let Some(isbn) = literature.isbn {
            lit.push_str(", ");
            lit.push_str(&extract_plain_text(isbn));
        }
        if let Some(pages) = literature.pages {
            lit.push_str(", ");
            lit.push_str(&format!("S. {}", &extract_plain_text(pages)));
        }
        lit.push_str(".");
        write!(out, "{}", lit)
    }

    fn question(
        &mut self,
        question: &Question<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let title = match question.kind {
            Some(e) => e.render(self, settings)?,
            None => String::new(),
        };
        let question_text = question.question.render(self, settings)?;
        let answer = question.answer.render(self, settings)?;
        self.environment(
            "question",
            &[title.trim()],
            &format!("{}\n\n{}", question_text.trim(), answer.trim()),
            out,
        )
    }

    fn proof_by_cases(
        &mut self,
        cases: &ProofByCases<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<()> {
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
                let goal = case.render(self, settings)?;
                writeln!(out, PROOF_CASE_CAPTION!(), "Fall", index + 1, goal.trim())?;
                self.run_vec(&proof, settings, out)?;
            }
        }
        Ok(())
    }

    fn group_exercise(
        &mut self,
        group: &GroupExercise<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let title = group.title.unwrap_or(&[]).render(self, settings)?;

        let tasks;
        let solutions;
        {
            let mut build_items = |solution| -> Vec<String> {
                group
                    .present
                    .iter()
                    .filter(|a| {
                        a.name.starts_with("subtask")
                            && (if solution {
                                a.name.ends_with("solution")
                            } else {
                                !a.name.ends_with("solution")
                            })
                    }).map(|a| {
                        let mut s = "\\item ".to_string();
                        s.push_str(
                            &a.value
                                .render(self, settings)
                                .expect("unexpected error during latex rendering!")
                                .trim(),
                        );
                        s
                    }).collect()
            };

            let task_list = build_items(false);
            let solution_list = build_items(true);
            tasks = format!(LIST!(), "enumerate", &task_list.join("\n"), "enumerate");
            solutions = format!(LIST!(), "enumerate", &solution_list.join("\n"), "enumerate");
        }

        let mut exercise = if let Some(exercise_raw) = group.exercise {
            let exercise = exercise_raw.render(self, settings)?;
            format!(EXERCISE_TASKLIST!(), exercise.trim(), tasks.trim())
        } else {
            String::new()
        };

        if let Some(explanation) = group.explanation {
            let exp = explanation.render(self, settings)?;
            exercise = format!(EXERCISE_EXPLANATION!(), exercise.trim(), exp.trim());
        }

        self.environment("exercise", &[title.trim()], exercise.trim(), out)?;
        self.environment("solution", &[title.trim()], solutions.trim(), out)?;
        Ok(())
    }

    fn induction(
        &mut self,
        induction: &Induction<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let basic = if let Some(e) = induction.basic_set {
            e.render(self, settings)?
        } else {
            INDUCTION_SET_DEFAULT!().to_string()
        };
        let statement = induction.statement.render(self, settings)?;
        let base_case = induction.base_case.render(self, settings)?;
        let hypothesis = induction.induction_hypothesis.render(self, settings)?;
        let step_case_goal = induction.step_case_goal.render(self, settings)?;
        let step_case = induction.step_case.render(self, settings)?;
        writeln!(
            out,
            INDUCTION!(),
            basic.trim(),
            statement.trim(),
            base_case.trim(),
            hypothesis.trim(),
            step_case_goal.trim(),
            step_case.trim()
        )
    }

    fn important(
        &mut self,
        settings: &'s Settings,
        template: &Important<'e>,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let content = template.content.render(self, settings)?;
        self.environment(IMPORTANT_ENV!(), &[], content.trim(), out)
    }

    fn anchor(
        &self,
        root: &'e KnownTemplate,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<()> {
        if let Some(anchor) = extract_template_anchor(root, settings) {
            write!(out, LABEL!(), base64::encode(&anchor))?;
        } else {
            self.write_error("anchor export could not extract an anchor?", out)?;
        }
        Ok(())
    }

    fn mainarticle(
        &mut self,
        settings: &'s Settings,
        template: &Mainarticle<'e>,
        out: &mut io::Write,
    ) -> io::Result<()> {
        write!(out, MAINARTICLE!())?;
        let caption = extract_plain_text(&template.article);
        let mut target = "Mathe für Nicht-Freaks: ".to_string();
        target.push_str(&caption);

        self.internal_link(&target, &caption, settings, out)?;
        writeln!(out, "\\\\");
        Ok(())
    }

    pub fn template_arg(
        &mut self,
        root: &'e TemplateArgument,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        self.run_vec(&root.value, settings, out)?;
        Ok(false)
    }

    pub fn environment_template(
        &mut self,
        settings: &'s Settings,
        template: &KnownTemplate<'e>,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let title = template.find("title").map(|a| a.value).unwrap_or(&[]);
        let title_text = title.render(self, settings)?;

        if let Some(anchor) = extract_template_anchor(template, settings) {
            writeln!(out, LABEL!(), base64::encode(&anchor))?
        }

        for attribute in template.present() {
            if attribute.name == "title" {
                continue;
            }

            let content = attribute.value.render(self, settings)?;
            self.environment(&attribute.name, &[title_text.trim()], content.trim(), out)?;
        }
        Ok(())
    }
}
