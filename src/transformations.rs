use mediawiki_parser::ast::*;
use mediawiki_parser::transformations::*;
use settings::Settings;
use util::*;


/// Convert template name paragraphs to lowercase text only.
pub fn normalize_template_names(mut root: Element, settings: &Settings) -> TResult {
    if let &mut Element::Template { ref mut name, ref position, .. } = &mut root {

        let new_text = match name.drain(..).next() {
            Some(Element::Paragraph { content, .. }) => {
                content
            },
            Some(e) => { vec![e] },
            None => { return Ok(Element::Error {
                        position: position.clone(),
                        message: "MFNF template name must not be empty!".to_string(),
                    })
            }
        };

        if let Some(&Element::Text { ref position, ref text }) = new_text.first() {
            name.clear();
            name.push(
                Element::Text {
                    position: position.clone(),
                    text: if text.starts_with("#") {
                                text.clone()
                            } else {
                                // convert to lowercase and remove prefixes
                                let mut temp_text = &text.to_lowercase()[..];
                                for prefix in &settings.template_prefixes[..] {
                                    temp_text = trim_prefix(temp_text, prefix);
                                }
                                String::from(temp_text)
                            },
                }
            );
        } else {
            return Ok(Element::Error {
                position: if let Some(e) = new_text.first() {
                    e.get_position().clone()
                } else {
                    position.clone()
                },
                message: "MFNF Template names must be plain strings \
                        With no markup!".to_string(),
            });
        }
    };
    recurse_inplace(&normalize_template_names, root, settings)
}

/// Translate template names and template attribute names.
pub fn translate_templates(mut root: Element, settings: &Settings) -> TResult {
    if let &mut Element::Template { ref mut name, ref mut content, .. } = &mut root {
        if let Some(&mut Element::Text { ref mut text, .. }) = name.first_mut() {
            if let Some(translation) = settings.translations.get(text) {
                text.clear();
                text.push_str(translation);
            }
        }
    }
    recurse_inplace(&translate_templates, root, settings)
}

/// Transform a formula template argument to text-only.
pub fn normalize_formula(mut root: Element, settings: &Settings) -> TResult {
    if let &mut Element::Template { ref name, ref mut content, ref position, .. } = &mut root {
        if let Some(&Element::Text {ref text, .. }) = name.first() {
            if text == "formula" {
                let arg_error = Element::Error {
                    position: position.clone(),
                    message: "Forumla templates must have exactly one anonymous argument, \
                                which is LaTeX source code entirely enclosed in <math></math>!".to_string(),
                };

                if content.len() != 1 {
                    return Ok(arg_error);
                }
                if let Some(&mut Element::TemplateArgument {ref mut value, .. }) = content.first_mut() {
                    if value.len() != 1 {
                        return Ok(arg_error);
                    }
                    if let Some(Element::Formatted { ref markup, ref mut content, .. }) = value.pop() {
                        if content.len() != 1 || if let &MarkupType::Math = markup {false} else {true} {
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
        }
    };
    recurse_inplace(&normalize_formula, root, settings)
}
