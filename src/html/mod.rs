use preamble::*;

use std::io;
mod renderer;

/// serialize to html
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct HTMLTarget {
    strings: HTMLStrings,
    with_todo: bool,
}

/// all user-facing static strings.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]

pub struct HTMLStrings {
    proofcase_caption: String,
    question_caption: String,
    proofstep_caption: String,
    definition_caption: String,
    theorem_caption: String,
    example_caption: String,
    exercise_caption: String,
    hint_caption: String,
    warning_caption: String,
    proof_caption: String,
    alternativeproof_caption: String,
    proofsummary_caption: String,
    solution_caption: String,
    solutionprocess_caption: String,
    solutionprocess_env_caption: String,
    summary_env_caption: String,
    explanation_env_caption: String,
    exercise_part_solution_caption: String,
    induction_intro_basicset: String,
    induction_intro_default: String,
    induction_base_case: String,
    induction_hypothesis: String,
    induction_step: String,
    induction_step_goal: String,
    induction_step_proof: String,
}

impl Default for HTMLStrings {
    fn default() -> HTMLStrings {
        HTMLStrings {
            proofcase_caption: "Fall".into(),
            question_caption: "Frage".into(),
            proofstep_caption: "Beweisschritt".into(),
            definition_caption: "Definition".into(),
            theorem_caption: "Satz".into(),
            example_caption: "Beispiel".into(),
            exercise_caption: "Übung".into(),
            hint_caption: "Hinweis".into(),
            warning_caption: "Warnung".into(),
            proof_caption: "Beweis".into(),
            alternativeproof_caption: "Alternativer Beweis".into(),
            proofsummary_caption: "Beweiszusammenfassung".into(),
            solution_caption: "Lösung".into(),
            solutionprocess_caption: "Lösungsweg".into(),
            solutionprocess_env_caption: "Wie komme ich auf den Beweis?".into(),
            summary_env_caption: "Zusammenfassung".into(),
            explanation_env_caption: "Erklärung".into(),
            exercise_part_solution_caption: "Lösung von Teilaufgabe".into(),
            induction_intro_basicset: "Aussage, die wir für alle {} beweisen wollen: ".into(),
            induction_intro_default: "Aussage, die wir beweisen wollen: ".into(),
            induction_base_case: "Induktionsanfang:".into(),
            induction_step: "Induktionsschritt:".into(),
            induction_hypothesis: "Induktionsvoraussetzung:".into(),
            induction_step_goal: "Induktionsbehauptung:".into(),
            induction_step_proof: "Beweis des Induktionsschritts:".into(),
        }
    }
}

impl Target for HTMLTarget {
    fn include_sections(&self) -> bool {
        true
    }
    fn target_extension(&self) -> &str {
        "html"
    }
    fn extension_for(&self, ext: &str) -> &str {
        "%"
    }

    fn export<'a>(
        &self,
        root: &'a Element,
        settings: &Settings,
        _: &[String],
        out: &mut io::Write,
    ) -> io::Result<()> {
        let mut renderer = renderer::HtmlRenderer::new(self);
        renderer.run(root, settings, out)
    }
}
