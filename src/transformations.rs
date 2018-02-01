use mediawiki_parser::transformations::*;
use mediawiki_parser::TransformationError;
use mediawiki_parser::ListItemKind;
use preamble::*;
use std::fs::File;
use serde_yaml;


/// Convert template name paragraphs to lowercase text only.
pub fn normalize_template_names(mut root: Element, settings: &Settings) -> TResult {
    if let Element::Template { ref mut name, ref mut content, ref position, .. } = root {

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
            if let Element::TemplateArgument { ref mut name, .. } = *child {
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
                    text: if text.starts_with('#') {
                        String::from(text.trim())
                    } else {
                        // convert to lowercase and remove prefixes
                        let mut temp_text = &text.trim().to_lowercase()[..];
                        let prefixes = &settings.template_prefixes;
                        for prefix in prefixes {
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
    if let Element::Template { ref mut name, ref mut content, .. } = root {
        let translation_tab = &settings.translations;
        if let Some(&mut Element::Text { ref mut text, .. }) = name.first_mut() {
            if let Some(translation) = translation_tab.get(text) {
                text.clear();
                text.push_str(translation);
            }
        }
        for child in content {
            if let Element::TemplateArgument { ref mut name, .. } = *child {
                if let Some(translation) = translation_tab.get(name) {
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
    if let Element::TemplateArgument { ref name, ref mut value, ref position } = root {
        if name == "title" {
            let mut last_value = value.pop();
            // title is empty
            if last_value.is_none() {
                return Err(TransformationError {
                    cause: "A template title must not be empty!".to_string(),
                    position: position.clone(),
                    transformation_name: "normalize_template_title".to_string(),
                    tree: Element::TemplateArgument {
                        name: name.clone(),
                        value: vec![],
                        position: position.clone(),
                    }
                })
            }
            if let Some(Element::Paragraph { ref mut content, .. }) = last_value {
                if let Some(&Element::Text { ref text, ref position  }) = content.last() {
                    value.clear();
                    value.push(Element::Text {
                        text: String::from(text.trim()),
                        position: position.clone(),
                    });
                }
            } else {
                value.push(last_value.unwrap());
            }
        }
    }
    recurse_inplace(&normalize_template_title, root, settings)
}


pub fn include_sections(
    mut root: Element,
    settings: &Settings) -> TResult {
    root = recurse_inplace_template(&include_sections, root, settings, &include_sections_vec)?;
    Ok(root)
}

pub fn include_sections_vec<'a>(
    trans: &TFuncInplace<&'a Settings>,
    root_content: &mut Vec<Element>,
    settings: &'a Settings) -> TListResult {

    // search for section inclusion in children
    let mut result = vec![];
    for mut child in root_content.drain(..) {

        if let Element::Template {
            ref name,
            ref content,
            ref position
        } = child {
            let prefix = &settings.section_inclusion_prefix;
            let template_name = extract_plain_text(name);

            // section transclusion
            if template_name.to_lowercase().trim().starts_with(prefix) {
                let article = trim_prefix(template_name.trim(), prefix);
                if content.len() < 1 {
                    return Err(TransformationError {
                        cause: "A section inclusion must specify article \
                                name and section name!".to_string(),
                        position: position.clone(),
                        transformation_name: "include_sections".to_string(),
                        tree: Element::Template {
                            name: name.clone(),
                            position: position.clone(),
                            content: content.clone(),
                        }
                    });
                }

                let section_name = extract_plain_text(content);
                let path = get_section_path(article, &section_name, settings);

                // error returned when the section file is faulty
                let file_error = TransformationError {
                    cause: format!("section file `{}` could not \
                                   be read or parsed!", &path),
                    position: position.clone(),
                    transformation_name: "include_sections".to_string(),
                    tree: Element::Template {
                        name: name.clone(),
                        position: position.clone(),
                        content: content.clone(),
                    }
                };

                let section_str = File::open(&path);
                if section_str.is_err() {
                    return Err(file_error)
                }

                let mut section_tree: Vec<Element>
                    = match serde_yaml::from_reader(&section_str.unwrap()) {
                    Ok(root) => root,
                    Err(_) => return Err(file_error),
                };

                result.push(
                    Element::Comment {
                        position: position.clone(),
                        text: format!("included from: {}|{}", article, section_name),
                    }
                );

                // recursively include sections
                // heading depths are normalized in a later transformation
                section_tree = include_sections_vec(
                    &include_sections,
                    &mut section_tree,
                    settings,
                )?;
                result.append(&mut section_tree);
                continue
            }
        }
        result.push(trans(child, settings)?);
    }
    Ok(result)
}

/// Normalize heading depths by making subheadings one level deeper than their parent.
/// The highest level of headings is assigned depth 1.
pub fn normalize_heading_depths(
    mut root: Element,
    _settings: &Settings) -> TResult {
    root = normalize_heading_depths_traverse(root, 1)?;
    Ok(root)
}

fn normalize_heading_depths_traverse(
    mut root: Element,
    current_depth: usize) -> TResult {

    let mut current_depth = current_depth;

    if let Element::Heading { ref mut depth, .. } = root {
        *depth = current_depth;
        current_depth += 1;
    }

    recurse_inplace(&normalize_heading_depths_traverse, root, current_depth)
}

/// Remove prefixes before filenames of included files.
pub fn remove_file_prefix(mut root: Element, settings: &Settings) -> TResult {
    if let Element::InternalReference { ref mut target, .. } = root {
        if let Some(&mut Element::Text { ref mut text, .. }) = target.first_mut() {
            for prefix in &settings.file_prefixes {
                let prefix_str = format!("{}:", &prefix);
                let new_text = String::from(trim_prefix(text, &prefix_str));
                text.clear();
                text.push_str(&new_text);
            }
        }
    }
    recurse_inplace(&remove_file_prefix, root, settings)
}

/// Convert list templates (MFNF) to mediawiki lists.
pub fn convert_template_list(mut root: Element, settings: &Settings) -> TResult {
    if let Element::Template { ref name, ref mut content, ref position } = root {
        if extract_plain_text(name) == "list" {

            let mut list_content = vec![];

            let list_type = if let Some(&Element::TemplateArgument {
                ref value,
                ..
            }) = find_arg(content, "type") {
                extract_plain_text(value).to_lowercase()
            } else {
                String::new()
            };

            let item_kind = match list_type.trim() {
                "ol" | "ordered" => ListItemKind::Ordered,
                "ul" | _ => ListItemKind::Unordered,
            };

            for child in content.drain(..) {
                if let Element::TemplateArgument {
                    name,
                    mut value,
                    position,
                } = child {
                    if name.starts_with("item") {
                        let li = Element::ListItem {
                            position,
                            content: value,
                            kind: item_kind,
                            depth: 1,
                        };
                        list_content.push(li);
                    // a whole sublist only wrapped by the template,
                    // -> replace template by wrapped list
                    } else if name.starts_with("list") {
                        if value.is_empty() {
                            continue
                        }
                        let sublist = value.remove(0);
                        return recurse_inplace(&convert_template_list, sublist, settings);
                    }
                }
            }

            let list = Element::List {
                position: position.to_owned(),
                content: list_content,
            };
            return recurse_inplace(&convert_template_list, list, settings);
        }
    }
    recurse_inplace(&convert_template_list, root, settings)
}
