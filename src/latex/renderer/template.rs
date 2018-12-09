//! Implements template rendering for latex.

use super::LatexRenderer;
use crate::anchors::extract_template_anchor;
use crate::preamble::*;
use base64;
use mfnf_template_spec::*;
use mwparser_utils::*;

impl<'e, 's: 'e, 't: 'e, 'a> LatexRenderer<'e, 't, 's, 'a> {
    pub fn template(&mut self, root: &'e Template, out: &mut io::Write) -> io::Result<bool> {
        let doctitle = &self.args.document_title;
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
            KnownTemplate::Formula(formula) => self.formula(&formula, out)?,
            KnownTemplate::Important(important) => self.important(&important, out)?,
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
            | KnownTemplate::SolutionProcess(_) => self.environment_template(&parsed, out)?,
            KnownTemplate::GroupExercise(group) => self.group_exercise(&group, out)?,
            KnownTemplate::ProofStep(step) => self.proofstep(&step, out)?,
            KnownTemplate::Anchor(_) => self.anchor(&parsed, out)?,
            KnownTemplate::Mainarticle(article) => self.mainarticle(&article, out)?,
            KnownTemplate::Navigation(_) => (),
            KnownTemplate::Question(question) => self.question(&question, out)?,
            KnownTemplate::ProofByCases(cases) => self.proof_by_cases(&cases, out)?,
            KnownTemplate::Induction(induction) => self.induction(&induction, out)?,
            KnownTemplate::Smiley(smiley) => write!(
                out,
                "{}",
                smiley_to_unicode(&extract_plain_text(&smiley.name.unwrap_or(&[])))
                    .unwrap_or('\u{01f603}')
            )?,
            // TODO: replace noprint with a sematic version, ignore for now.
            KnownTemplate::NoPrint(noprint) => {
                if self.latex.with_todo {
                    self.run_vec(&noprint.content, (), out)?
                }
            }
            KnownTemplate::Todo(todo) => self.todo(&todo, out)?,
        };
        Ok(false)
    }

    fn formula(&self, formula: &Formula, out: &mut io::Write) -> io::Result<()> {
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
            })
            .next();

        if let Some(err) = error {
            self.error(err, out)?;
            return Ok(());
        }

        let content = extract_plain_text(formula.formula).trim().to_owned();

        let mut trimmed = trim_enclosing(&content, "\\begin{align}", "\\end{align}");
        trimmed = trim_enclosing(trimmed, "\\begin{align*}", "\\end{align*}").trim();
        self.environment(MATH_ENV!(), &[], trimmed, out)
    }

    fn proofstep(&mut self, step: &ProofStep<'e>, out: &mut io::Write) -> io::Result<()> {
        let name = match step.name {
            Some(name) => name.render(self)?,
            None => "Beweisschritt".into(),
        };
        let goal = step.goal.render(self)?;
        let step = step.step.render(self)?;
        let separator = &self.latex.paragraph_separator;
        let body = format!("{}{}\n{}", goal.trim(), separator, step);
        self.environment(PROOF_STEP_ENV!(), &[name.trim()], body.trim(), out)
    }

    fn todo(&mut self, todo: &Todo<'e>, out: &mut io::Write) -> io::Result<()> {
        if self.latex.with_todo {
            let text = todo.todo.render(self)?;
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

    fn question(&mut self, question: &Question<'e>, out: &mut io::Write) -> io::Result<()> {
        let title = match question.kind {
            Some(e) => e.render(self)?,
            None => String::new(),
        };
        let question_text = question.question.render(self)?;
        let answer = question.answer.render(self)?;
        let sep = &self.latex.paragraph_separator;
        self.environment(
            "question",
            &[title.trim()],
            &format!("{}{}\n{}", question_text.trim(), sep, answer.trim()),
            out,
        )
    }

    fn proof_by_cases(&mut self, cases: &ProofByCases<'e>, out: &mut io::Write) -> io::Result<()> {
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
                let goal = case.render(self)?;
                let proof = proof.render(self)?;
                let name = format!("Fall {}", index + 1);
                let sep = &self.latex.paragraph_separator;
                let content = format!("{}{}\n{}", goal.trim(), sep, proof.trim());
                self.environment(PROOF_CASE_ENV!(), &[&name], &content, out)?;
            }
        }
        Ok(())
    }

    fn group_exercise(&mut self, group: &GroupExercise<'e>, out: &mut io::Write) -> io::Result<()> {
        let title = group.title.unwrap_or(&[]).render(self)?;

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
                    })
                    .map(|a| {
                        let mut s = "\\item ".to_string();
                        s.push_str(
                            &a.value
                                .render(self)
                                .expect("unexpected error during latex rendering!")
                                .trim(),
                        );
                        s
                    })
                    .collect()
            };

            let task_list = build_items(false);
            let solution_list = build_items(true);
            tasks = format!(LIST!(), "enumerate", &task_list.join("\n"), "enumerate");
            solutions = format!(LIST!(), "enumerate", &solution_list.join("\n"), "enumerate");
        }

        let mut exercise = if let Some(exercise_raw) = group.exercise {
            let exercise = exercise_raw.render(self)?;
            format!(EXERCISE_TASKLIST!(), exercise.trim(), tasks.trim())
        } else {
            String::new()
        };

        if let Some(explanation) = group.explanation {
            let exp = explanation.render(self)?;
            exercise = format!(EXERCISE_EXPLANATION!(), exercise.trim(), exp.trim());
        }

        self.environment("exercise", &[title.trim()], exercise.trim(), out)?;
        self.environment("solution", &[title.trim()], solutions.trim(), out)?;
        Ok(())
    }

    fn induction(&mut self, induction: &Induction<'e>, out: &mut io::Write) -> io::Result<()> {
        let basic = if let Some(e) = induction.basic_set {
            e.render(self)?
        } else {
            INDUCTION_SET_DEFAULT!().to_string()
        };
        let statement = induction.statement.render(self)?;
        let base_case = induction.base_case.render(self)?;
        let hypothesis = induction.induction_hypothesis.render(self)?;
        let step_case_goal = induction.step_case_goal.render(self)?;
        let step_case = induction.step_case.render(self)?;
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

    fn important(&mut self, template: &Important<'e>, out: &mut io::Write) -> io::Result<()> {
        let content = template.content.render(self)?;
        self.environment(IMPORTANT_ENV!(), &[], content.trim(), out)
    }

    fn anchor(&self, root: &'e KnownTemplate, out: &mut io::Write) -> io::Result<()> {
        let doctitle = &self.args.document_title;
        if let Some(anchor) = extract_template_anchor(root, doctitle) {
            write!(out, LABEL!(), base64::encode(&anchor))?;
        } else {
            self.write_error("anchor export could not extract an anchor?", out)?;
        }
        Ok(())
    }

    fn mainarticle(&mut self, template: &Mainarticle<'e>, out: &mut io::Write) -> io::Result<()> {
        write!(out, MAINARTICLE!())?;
        let caption = extract_plain_text(&template.article);
        let mut target = "Mathe für Nicht-Freaks: ".to_string();
        target.push_str(&caption);

        self.internal_link(&target, &caption, out)?;
        writeln!(out, "\n")
    }

    pub fn template_arg(
        &mut self,
        root: &'e TemplateArgument,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        self.run_vec(&root.value, (), out)?;
        Ok(false)
    }

    pub fn environment_template(
        &mut self,
        template: &KnownTemplate<'e>,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let title = template.find("title").map(|a| a.value).unwrap_or(&[]);
        let title_text = title.render(self)?;
        let doctitle = &self.args.document_title;

        if let Some(anchor) = extract_template_anchor(template, doctitle) {
            write!(out, LABEL!(), base64::encode(&anchor))?;
            writeln!(out, "%")?
        }

        for attribute in template.present() {
            if attribute.name == "title" {
                continue;
            }

            let content = attribute.value.render(self)?;
            self.environment(&attribute.name, &[title_text.trim()], content.trim(), out)?;
        }
        Ok(())
    }
}
