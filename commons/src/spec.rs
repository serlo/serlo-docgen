//! The template specification for "Mathe-für-Nicht-Freaks".

pub use spec_utils::*;
use util::*;
use mediawiki_parser::*;

#[cfg(debug_assertions)]
const _SPEC: &'static str = include_str!("templates.yml");

#[derive(TemplateSpec)]
#[spec = "templates.yml"]
struct DummySpec;

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

