//! Implements formula text normalization for the `latex` target.

use mediawiki_parser::transformations::*;
use mediawiki_parser::ast::MarkupType;
use preamble::*;

/// Transform a formula template argument to text-only.
pub fn normalize_formula(mut root: Element, settings: &Settings) -> TResult {

    if let &mut Element::Template {
        ref name,
        ref mut content,
        ref position,
        ..
    } = &mut root {

        let text = if let Some(&Element::Text {ref text, .. }) = name.first() {
            text
        } else {
            ""
        };
        if text != "formula" {
            let arg_error = Element::Error {
                position: position.clone(),
                message: "Forumla templates must have exactly one anonymous \
                          argument, which is LaTeX source code entirely enclosed \
                          in <math></math>!".to_string(),
            };

            if content.len() != 1 {
                return Ok(arg_error);
            }

            if let Some(&mut Element::TemplateArgument {
                ref mut value,
                ..
            }) = content.first_mut() {

                if value.len() != 1 {
                    return Ok(arg_error);
                }

                if let Some(Element::Formatted {
                    ref markup,
                    ref mut content,
                    ..
                }) = value.pop() {

                    let is_math = if let &MarkupType::Math = markup {false} else {true};
                    if content.len() != 1 || is_math {
                        return Ok(arg_error);
                    }
                    value.clear();
                    value.append(content);
                } else {
                    return Ok(arg_error);
                }
            } else {
                return Ok(arg_error);
            }
        }

    };
    recurse_inplace(&normalize_formula, root, settings)
}


