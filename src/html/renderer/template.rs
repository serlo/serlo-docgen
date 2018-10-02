use super::HtmlRenderer;
use mfnf_template_spec::*;
use mwparser_utils::*;
use preamble::*;

macro_rules! tag_wrapper {
    ($self:ident, $content:expr, $settings:ident, $out:ident, $tag:expr, $class:expr) => {
        write!($out, "<{} class=\"{}\">", $tag, $class)?;
        $self.run_vec($content, $settings, $out)?;
        write!($out, "</{}>", $tag)?;
    };
}

macro_rules! tag_stmt {
    ($content:stmt, $out:expr, $tag:expr, $class:expr) => {
        write!($out, "<{} class=\"{}\">", $tag, $class)?;
        $content
        write!($out, "</{}>", $tag)?;
    };
}

macro_rules! tag_str {
    ($content:expr, $out:expr, $tag:expr, $class:expr) => {
        tag_stmt!(write!($out, "{}", $content)?, $out, $tag, $class)
    };
}

macro_rules! div_wrapper {
    ($self:ident, $content:expr, $settings:ident, $out:ident, $class:expr) => {
        tag_wrapper!($self, $content, $settings, $out, "div", $class)
    };
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
            KnownTemplate::GroupExercise(group) => self.group_exercise(&group, settings, out)?,
            KnownTemplate::NoPrint(noprint) => {
                self.run_vec(&noprint.content, settings, out)?;
                false
            }
            KnownTemplate::Navigation(_) => false,
            KnownTemplate::Important(important) => self.important(settings, &important, out)?,
            KnownTemplate::Todo(_) => false,
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
                let text = extract_plain_text(&smiley.name.unwrap_or(&[]));
                let unicode = smiley_to_unicode(&text).unwrap_or('\u{01f603}');

                write!(out, "{}", &unicode)?;
                false
            }
            KnownTemplate::Anchor(_) => {
                self.write_error("TODO", out)?;
                false
            }
            KnownTemplate::Mainarticle(_) => {
                self.write_error("TODO", out)?;
                false
            }
            KnownTemplate::Literature(_) => {
                self.write_error("TODO", out)?;
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
                writeln!(
                    out,
                    "<span class=\"proofcase\">{} {}:</span>",
                    self.html.strings.proofcase_caption,
                    index + 1
                )?;
                div_wrapper!(self, &case, settings, out, "proofcase-case");
                div_wrapper!(self, &proof, settings, out, "proofcase-proof");
            }
        }
        Ok(false)
    }
    fn important(
        &mut self,
        settings: &'s Settings,
        template: &Important<'e>,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        div_wrapper!(self, &template.content, settings, out, "important");
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
            }).next();

        if let Some(err) = error {
            self.error(err, out)?;
            return Ok(false);
        }
        tag_stmt!(
            {
                let refs: Vec<&Element> = formula.formula.iter().collect();
                match refs[..] {
                    [&Element::Formatted(ref root)] => {
                        if let MarkupType::Math = root.markup {
                            self.formel(root, settings, out)?;
                        } else {
                            let msg = format!(
                                "the first element of the content of \"formula\" \
                                 is not math, but {:?}!",
                                root.markup
                            );
                            self.write_error(&msg, out)?;
                        }
                    }
                    _ => {
                        let msg = format!(
                            "the content of \"formula\" is not \
                             only a math element, but {:?}!",
                            formula.formula
                        );
                        self.write_error(&msg, out)?;
                    }
                }
            },
            out,
            "div",
            "formula"
        );
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
            tag_stmt!(
                {
                    self.run_vec(&kind, settings, out)?;
                    write!(out, ": ")?;
                },
                out,
                "span",
                "question-caption"
            );
        } else {
            let caption = format!("{}: ", &self.html.strings.question_caption);
            tag_str!(&caption, out, "span", "question-caption");
        }
        tag_wrapper!(
            self,
            &question.question,
            settings,
            out,
            "span",
            "question-text"
        );
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
        tag_stmt!(
            {
                let caption = format!("{}: ", &self.html.strings.proofstep_caption);
                tag_str!(&caption, out, "span", "proofstep-caption");
                tag_wrapper!(self, &step.goal, settings, out, "span", "proofstep-goal");
            },
            out,
            "summary",
            "proofstep"
        );
        div_wrapper!(self, &step.step, settings, out, "proofstep");
        write!(out, "</details>")?;
        Ok(false)
    }

    pub fn environment_template(
        &mut self,
        template: &KnownTemplate<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
        class: &str,
    ) -> io::Result<bool> {
        write!(out, "<div class=\"{} environment\">", class)?;
        write!(out, "<div class=\"icon icon-{}\">", class)?;
        let name = match template {
            KnownTemplate::Definition(_) => &self.html.strings.definition_caption,
            KnownTemplate::Theorem(_) => &self.html.strings.theorem_caption,
            KnownTemplate::Example(_) => &self.html.strings.example_caption,
            KnownTemplate::Exercise(_) => &self.html.strings.exercise_caption,
            KnownTemplate::Hint(_) => &self.html.strings.hint_caption,
            KnownTemplate::Warning(_) => &self.html.strings.warning_caption,
            KnownTemplate::Proof(_) => &self.html.strings.proof_caption,
            KnownTemplate::AlternativeProof(_) => &self.html.strings.alternativeproof_caption,
            KnownTemplate::ProofSummary(_) => &self.html.strings.proofsummary_caption,
            KnownTemplate::Solution(_) => &self.html.strings.solution_caption,
            KnownTemplate::SolutionProcess(_) => &self.html.strings.solutionprocess_caption,
            _ => "Unknown Template",
        };
        write!(out, "{}: ", &name)?;

        let title = template.find("title");
        if let Some(render_title) = title {
            tag_wrapper!(
                self,
                &render_title.value,
                settings,
                out,
                "span",
                "environment-title"
            );
        }
        for attribute in template.present() {
            if attribute.name == "title" {
                continue;
            }
            let class_attribute = format!("env-{}", &attribute.name);
            let icon_name = format!("icon icon-{}", &attribute.name);
            let class_title = format!("title-env-{}", &attribute.name);
            let attribute_name = match attribute.name.as_ref() {
                "example" => &self.html.strings.example_caption,
                "solutionprocess" => &self.html.strings.solutionprocess_env_caption,
                "summary" => &self.html.strings.summary_env_caption,
                "proof" => &self.html.strings.proof_caption,
                "explanation" => &self.html.strings.explanation_env_caption,
                _ => "",
            };
            tag_stmt!(
                {
                    if attribute.name.to_string() == class.to_string() {
                        tag_str!(&attribute_name, out, "span", &class_title);
                        self.run_vec(&attribute.value, settings, out)?;
                    }
                    //catches the case, that the attribute has the same name as the type and so that the icon is rendered two times
                    else {
                        tag_stmt!(
                            {
                                tag_str!(&attribute_name, out, "span", &class_title);
                                self.run_vec(&attribute.value, settings, out)?;
                            },
                            out,
                            "div",
                            &icon_name
                        );
                    }
                },
                out,
                "div",
                &class_attribute
            );
        }
        write!(out, "</div>")?;
        write!(out, "</div>")?;
        Ok(false)
    }
    fn group_exercise(
        &mut self,
        group: &GroupExercise<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        tag_stmt!(
            {
                if let Some(render_title) = &group.title {
                    div_wrapper!(self, &render_title, settings, out, "exercise-title");
                };
                if let Some(exercise) = &group.exercise {
                    div_wrapper!(self, &exercise, settings, out, "exercise-content");
                };
                if let Some(explanation) = &group.explanation {
                    div_wrapper!(self, &explanation, settings, out, "exercise-explanation");
                };
                let subtaskts = [
                    group.subtask1,
                    group.subtask2,
                    group.subtask3,
                    group.subtask4,
                    group.subtask5,
                    group.subtask6,
                ];
                let solutions = [
                    group.subtask1_solution,
                    group.subtask2_solution,
                    group.subtask3_solution,
                    group.subtask4_solution,
                    group.subtask5_solution,
                    group.subtask6_solution,
                ];
                for (index, item) in subtaskts.iter().enumerate() {
                    if let Some(subtask) = item {
                        let caption =
                            format!("{} {}: ", &self.html.strings.exercise_caption, index + 1);
                        tag_str!(&caption, out, "span", "exercise-exercise-caption");
                        div_wrapper!(self, &subtask, settings, out, "exercise-exercise");
                    }
                }
                write!(out, "<details open class =\"exercise-solution-container\">")?;
                let solution_caption = format!("{}: ", &self.html.strings.solution_caption);
                tag_str!(
                    &solution_caption,
                    out,
                    "summary",
                    "group_exercise-solution-title"
                );
                for (index, item) in solutions.iter().enumerate() {
                    if let Some(solution) = item {
                        let caption = format!(
                            "{} {}: ",
                            &self.html.strings.exercise_part_solution_caption,
                            index + 1
                        );
                        tag_str!(&caption, out, "span", "exercise-solution-caption");
                        div_wrapper!(self, &solution, settings, out, "exercise-solution");
                    }
                }
                write!(out, "</details>")?;
            },
            out,
            "div",
            "group_exercise"
        );
        Ok(false)
    }

    pub fn solution(
        &mut self,
        solution: &Solution<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        tag_stmt!(
            {
                tag_stmt!(
                    {
                        tag_stmt!(
                            {
                                let caption = format!("{}: ", &self.html.strings.solution_caption);
                                tag_str!(&caption, out, "span", "solution-caption");
                                if let Some(render_title) = &solution.title {
                                    self.run_vec(&render_title, settings, out)?;
                                }
                            },
                            out,
                            "summary",
                            "solution-summary"
                        );
                        self.run_vec(&solution.solution, settings, out)?;
                    },
                    out,
                    "details",
                    "solution"
                );
            },
            out,
            "div",
            "icon icon-solution"
        );
        Ok(false)
    }

    fn induction(
        &mut self,
        induction: &Induction<'e>,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        let strings = &self.html.strings;

        write!(out, "<div class=\"induction\">")?;
        if let Some(e) = induction.basic_set {
            let set = e.render(self, settings)?;
            let msg = str::replace(&strings.induction_intro_basicset, "{}", &set);
            tag_str!(&msg, out, "span", "induction-intro");
        } else {
            tag_str!(
                &strings.induction_intro_default,
                out,
                "span",
                "induction-intro"
            );
        };
        self.run_vec(&induction.statement, settings, out)?;

        tag_stmt!(
            {
                write!(out, "<li><details open><summary>")?;
                tag_str!(
                    &strings.induction_base_case,
                    out,
                    "span",
                    "induction-base-caption"
                );
                write!(out, "</summary>")?;
                self.run_vec(&induction.base_case, settings, out)?;
                write!(out, "</details></li>")?;

                write!(out, "<li><details open><summary>")?;
                tag_str!(
                    &strings.induction_step,
                    out,
                    "span",
                    "induction-step-caption"
                );
                write!(out, "</summary>")?;

                tag_stmt!(
                    {
                        write!(out, "<li><details open><summary>")?;
                        tag_str!(
                            &strings.induction_hypothesis,
                            out,
                            "span",
                            "induction-hypothesis-caption"
                        );
                        write!(out, "</summary>")?;
                        self.run_vec(&induction.induction_hypothesis, settings, out)?;
                        write!(out, "</details></li>")?;

                        write!(out, "<li><details open><summary>")?;
                        tag_str!(
                            &strings.induction_step_goal,
                            out,
                            "span",
                            "induction-step-goal-caption"
                        );
                        write!(out, "</summary>")?;
                        self.run_vec(&induction.step_case_goal, settings, out)?;
                        write!(out, "</details></li>")?;

                        write!(out, "<li><details open><summary>")?;
                        tag_str!(
                            &strings.induction_step_proof,
                            out,
                            "span",
                            "induction-step-proof-caption"
                        );
                        write!(out, "</summary>")?;
                        self.run_vec(&induction.step_case, settings, out)?;
                        write!(out, "</details></li>")?;
                    },
                    out,
                    "ol",
                    "induction-inner-list"
                );

                write!(out, "</details></li>")?;
            },
            out,
            "ol",
            "induction-outer-list"
        );

        write!(out, "</div>")?;

        Ok(false)
    }
}
