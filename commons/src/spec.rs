//! The template specification for "Mathe-für-Nicht-Freaks".

pub use spec_utils::*;
use util::*;
use mediawiki_parser::*;

template_spec!(
    template {
        name: "formula",
        alt: ["formel", "Formel", "Formula"],
        format: Format::Inline,
        attributes: [
            attribute!(
                name: "1",
                alt: ["formel"],
                priority: Priority::Required,
                predicate: &is_math_tag
            )
        ]
    },
    template {
        name: "important",
        alt: ["-"],
        format: Format::Block,
        attributes: [
            attribute!(
                name: "1",
                alt: ["content"],
                priority: Priority::Required,
                predicate: &is_text_only_paragraph
            )
        ]
    },
    template {
        name: "definition",
        alt: [":Mathe für Nicht-Freaks: Vorlage:Definition"],
        format: Format::Block,
        attributes: [
            attribute!(
                name: "title",
                alt: ["titel"],
                priority: Priority::Optional,
                predicate: &is_text_only_paragraph
            ),
            attribute!(
                name: "definition",
                alt: [],
                priority: Priority::Required,
                predicate: &is_text_only_paragraph
            )
        ]
    },
    template {
        name: "theorem",
        alt: [":Mathe für Nicht-Freaks: Vorlage:Satz"],
        format: Format::Block,
        attributes: [
            attribute!(
                name: "title",
                alt: ["titel"],
                priority: Priority::Optional,
                predicate: &is_text_only_paragraph
            ),
            attribute!(
                name: "theorem",
                alt: ["satz"],
                priority: Priority::Required,
                predicate: &is_text_only_paragraph
            )
        ]
    },
    template {
        name: "example",
        alt: [":Mathe für Nicht-Freaks: Vorlage:Beispiel"],
        format: Format::Block,
        attributes: [
            attribute!(
                name: "title",
                alt: ["titel"],
                priority: Priority::Optional,
                predicate: &is_text_only_paragraph
            ),
            attribute!(
                name: "example",
                alt: ["beispiel"],
                priority: Priority::Required,
                predicate: &is_text_only_paragraph
            )
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
