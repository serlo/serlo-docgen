use mediawiki_parser::ast::*;
use mediawiki_parser::transformations::*;
use settings::Settings;
use util::*;


/// Convert template name paragraphs to lowercase text only.
pub fn normalize_template_names(mut root: Element, settings: &Settings) -> TResult {
    if let &mut Element::Template { ref mut name, ref mut content, ref position, .. } = &mut root {

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

        for child in content {
            if let &mut Element::TemplateArgument { ref mut name, .. } = child {
                let lowercase = name.trim().to_lowercase();
                name.clear();
                name.push_str(&lowercase);
            } else {
                return Ok(Element::Error {
                    position: position.clone(),
                    message: "Only TemplateArguments are allowed as children of templates!".to_string(),
                })
            }
        }

        if let Some(&Element::Text { ref position, ref text }) = new_text.first() {
            name.clear();
            name.push(
                Element::Text {
                    position: position.clone(),
                    text: if text.starts_with("#") {
                                String::from(text.trim())
                            } else {
                                // convert to lowercase and remove prefixes
                                let mut temp_text = &text.trim().to_lowercase()[..];
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
        for child in content {
            if let &mut Element::TemplateArgument { ref mut name, .. } = child {
                if let Some(translation) = settings.translations.get(name) {
                    name.clear();
                    name.push_str(translation);
                }
            }
        }
    }
    recurse_inplace(&translate_templates, root, settings)
}

/// Convert template attribute `title` to text only.
pub fn normalize_template_title(mut root: Element, settings: &Settings) -> TResult {
    if let &mut Element::TemplateArgument { ref name, ref mut value, .. } = &mut root {
        if name == "title" {
            if let Some(Element::Paragraph { ref mut content, .. }) = value.pop() {
                if let Some(Element::Text { text, position  }) = content.pop() {
                    value.clear();
                    value.push(Element::Text {
                        text: String::from(text.trim()),
                        position
                    });
                }
            }
        }
    }
    recurse_inplace(&normalize_template_title, root, settings)
}


