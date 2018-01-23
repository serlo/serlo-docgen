use mediawiki_parser::ast::*;
use mediawiki_parser::transformations::*;
use mediawiki_parser::error::TransformationError;
use settings::*;
use util::*;
use std::path;
use std::collections::HashMap;
use std::fs::File;
use serde_yaml;
use config;

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
                        let prefixes: Vec<String> = setting!(settings.template_prefixes);
                        for prefix in prefixes {
                            temp_text = trim_prefix(temp_text, &prefix);
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
        let translation_tab: HashMap<String, String> = setting!(settings.translations);
        if let Some(&mut Element::Text { ref mut text, .. }) = name.first_mut() {
            if let Some(translation) = translation_tab.get(text) {
                text.clear();
                text.push_str(&translation);
            }
        }
        for child in content {
            if let &mut Element::TemplateArgument { ref mut name, .. } = child {
                if let Some(translation) = translation_tab.get(name) {
                    name.clear();
                    name.push_str(&translation);
                }
            }
        }
    }
    recurse_inplace(&translate_templates, root, settings)
}

/// Convert template attribute `title` to text only.
pub fn normalize_template_title(mut root: Element, settings: &Settings) -> TResult {
    if let &mut Element::TemplateArgument { ref name, ref mut value, ref position } = &mut root {
        if name == "title" {
            let mut last_value = value.pop();
            // title is empty
            if let None = last_value {
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

        if let &mut Element::Template {
            ref name,
            ref content,
            ref position
        } = &mut child {
            let prefix: String = setting!(settings.targets.deps.section_inclusion_prefix);
            let template_name = extract_plain_text(&name);

            // section transclusion
            if template_name.to_lowercase().trim().starts_with(&prefix) {
                let article = trim_prefix(template_name.trim(), &prefix);
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

                let mut section_file: String = setting!(settings.targets.deps.section_rev);
                let section_ext: String = setting!(settings.targets.deps.section_ext);
                let section_path: String = setting!(settings.targets.deps.section_path);
                let section_name = extract_plain_text(content);

                section_file.push('.');
                section_file.push_str(&section_ext);

                let path = path::Path::new(&section_path)
                    .join(&filename_to_make(&article))
                    .join(&filename_to_make(&section_name))
                    .join(&filename_to_make(&section_file));

                // error returned when the section file is faulty
                let file_error = TransformationError {
                    cause: format!("section file `{}` could not be read or parsed!",
                                &path.to_string_lossy()),
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

pub fn normalize_heading_depths_traverse(
    mut root: Element,
    current_depth: usize) -> TResult {

    let mut current_depth = current_depth;

    if let &mut Element::Heading { ref mut depth, .. } = &mut root {
        *depth = current_depth;
        current_depth += 1;
    }

    recurse_inplace(&normalize_heading_depths_traverse, root, current_depth)
}

/// Remove prefixes before filenames of included files.
pub fn remove_file_prefix(mut root: Element, settings: &Settings) -> TResult {
    if let &mut Element::InternalReference { ref mut target, .. } = &mut root {
        if let Some(&mut Element::Text { ref mut text, .. }) = target.first_mut() {
            let prefixes: Vec<String> = setting!(settings.file_prefixes);
            for prefix in prefixes {
                let prefix_str = format!("{}:", &prefix);
                let new_text = String::from(trim_prefix(text, &prefix_str));
                text.clear();
                text.push_str(&new_text);
            }
        }
    }
    recurse_inplace(&remove_file_prefix, root, settings)
}
