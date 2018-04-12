//! The template specification for "Mathe-für-Nicht-Freaks".

pub use spec_utils::*;
use util::*;
use mediawiki_parser::*;

template_spec!(
    template {
        id: Formula,
        names: ["formula", "formel"],
        format: Format::Inline,
        attributes: [
            {
                ident: formel,
                names: ["1", "formel"],
                priority: Priority::Required,
                predicate: &is_math_tag
            }
        ]
    },
    template {
        id: Anchor,
        names: ["anchor", "anker"],
        format: Format::Inline,
        attributes: [
            {
                ident: ref1,
                names: ["1"],
                priority: Priority::Required,
                predicate: &is_plain_text
            },
            {
                ident: ref2,
                names: ["2"],
                priority: Priority::Optional,
                predicate: &is_plain_text
            },
            {
                ident: ref3,
                names: ["3"],
                priority: Priority::Optional,
                predicate: &is_plain_text
            }
        ]
    },
    template {
        id: Mainarticle,
        names: [":Mathe für Nicht-Freaks: Vorlage:Hauptartikel"],
        format: Format::Inline,
        attributes: [
            {
                ident: article,
                names: ["1"],
                priority: Priority::Required,
                predicate: &is_plain_text
            }
        ]
    },
    template {
        id: Important,
        names: ["important", "-"],
        format: Format::Block,
        attributes: [
            {
                ident: content,
                names: ["1", "content"],
                priority: Priority::Required,
                predicate: &is_text_only_paragraph
            }
        ]
    },
    template {
        id: Definition,
        names: [":Mathe für Nicht-Freaks: Vorlage:Definition"],
        format: Format::Block,
        attributes: [
            {
                ident: title,
                names: ["title", "titel"],
                priority: Priority::Optional,
                predicate: &is_text_only_paragraph
            },
            {
                ident: definition,
                names: ["definition"],
                priority: Priority::Required,
                predicate: &is_text_only_paragraph
            }
        ]
    },
    template {
        id: Theorem,
        names: [":Mathe für Nicht-Freaks: Vorlage:Satz"],
        format: Format::Block,
        attributes: [
            {
                ident: title,
                names: ["title", "titel"],
                priority: Priority::Optional,
                predicate: &is_text_only_paragraph
            },
            {
                ident: theorem,
                names: ["theorem", "satz"],
                priority: Priority::Required,
                predicate: &is_text_only_paragraph
            },
            {
                ident: explanation,
                names: ["explanation", "erklärung"],
                priority: Priority::Optional,
                predicate: &is_text_only_paragraph
            },
            {
                ident: solutionprocess,
                names: ["solutionprocess", "lösungsweg"],
                priority: Priority::Optional,
                predicate: &is_text_only_paragraph
            },
            {
                ident: summary,
                names: ["summary", "zusammenfassung"],
                priority: Priority::Optional,
                predicate: &is_text_only_paragraph
            },
            {
                ident: proof,
                names: ["proof", "beweis"],
                priority: Priority::Optional,
                predicate: &is_text_only_paragraph
            }
        ]
    },
    template {
        id: Example,
        names: [":Mathe für Nicht-Freaks: Vorlage:Beispiel"],
        format: Format::Block,
        attributes: [
            {
                ident: title,
                names: ["title", "titel"],
                priority: Priority::Optional,
                predicate: &is_text_only_paragraph
            },
            {
                ident: example,
                names: ["example", "beispiel"],
                priority: Priority::Required,
                predicate: &is_text_only_paragraph
            }
        ]
    }
);

fn is_math_tag(elems: &[Element]) -> bool {
    if elems.len() != 1 {
        return false
    }
    if let Some(&Element::Formatted { ref markup, .. }) = elems.first() {
        *markup == MarkupType::Math
    } else {
        false
    }
}

/// Paragraphs or Text without any formatting or special contents.
pub fn is_plain_text(elems: &[Element]) -> bool {
    fn shallow(elements: &[Element]) -> bool {
        for elem in elements {
            let allowed = match *elem {
                Element::Paragraph { .. }
                | Element::Text { .. } => true,
                _ => false
            };
            if !allowed {
                return false
            }
        }
        true
    }
    TreeChecker::all(elems, &shallow)
}

fn is_text_only_paragraph(elems: &[Element]) -> bool {
    fn shallow(elements: &[Element]) -> bool {
        for elem in elements {
            match *elem {
                Element::Template { ref name, .. } => {
                    let name = extract_plain_text(name);
                    if let Some(spec) = spec_of(&name) {
                        if spec.format != Format::Inline {
                            return false
                        }
                    } else {
                        return false
                    }
                },
                Element::Gallery { .. }
                | Element::Heading { .. }
                | Element::Table { .. }
                | Element::TableRow { .. }
                | Element::TableCell { .. }
                | Element::InternalReference { .. }
                => return false,
                _ => (),
            }
        }
        true
    };
    TreeChecker::all(elems, &shallow)
}
