use mediawiki_parser::transformations::*;
use mediawiki_parser::TransformationError;
use mediawiki_parser::ListItemKind;
use mediawiki_parser::MarkupType;
use mediawiki_parser::Span;
use preamble::*;
use std::fs::File;
use std::process::Command;
use serde_yaml;


/// Convert template name paragraphs to lowercase text only.
pub fn normalize_template_names(mut root: Element, settings: &Settings) -> TResult {
    if let Element::Template { ref mut name, ref mut content, ref position, .. } = root {

        if name.is_empty() {
            return Ok(Element::Error {
                position: position.clone(),
                message: "MFNF template name must not be empty!".to_string(),
            })
        };

        let mut new_text = extract_plain_text(name).trim().to_owned();

        for child in content {
            if let Element::TemplateArgument { ref mut name, .. } = *child {
                let lowercase = name.trim().to_lowercase();
                name.clear();
                name.push_str(&lowercase);
            } else {
                return Ok(Element::Error {
                    position: position.clone(),
                    message: "Only TemplateArguments are allowed as \
                            children of templates!".into(),
                })
            }
        }

        if !new_text.is_empty() {

            // convert to lowercase and remove prefixes
            if !new_text.starts_with('#') {
                new_text = new_text.trim().to_lowercase();
            }

            let text = Element::Text {
                position: Span {
                    start: if let Some(e) = name.first() {
                        e.get_position().start.clone()
                    } else {
                        position.start.clone()
                    },
                    end: if let Some(e) = name.last() {
                        e.get_position().end.clone()
                    } else {
                        position.end.clone()
                    }
                },
                text: new_text,
            };
            name.clear();
            name.push(text);
        } else {
            return Ok(Element::Error {
                position: position.clone(),
                message: "Template names cannot be empty!".into(),
            });
        }
    };
    recurse_inplace(&normalize_template_names, root, settings)
}


/// Normalize math formulas with texvccheck
pub fn normalize_math_formulas(mut root: Element, settings: &Settings) -> TResult {

    if let Element::Formatted {
        ref markup,
        ref mut content,
        ref position,
    } = root {
        if *markup == MarkupType::Math {
            match check_formula(content, position, settings) {
                e @ Element::Text { .. } => {
                    content.clear();
                    content.push(e);
                },
                e => return Ok(e),
            }
        }
    }
    recurse_inplace(&normalize_math_formulas, root, settings)
}

/// Check a Tex formula, return normalized version or error
fn check_formula(
    content: &[Element],
    position: &Span,
    settings: &Settings
) -> Element {
    if content.len() != 1 {
        return Element::Error {
            message: "A formula must have exactly one content element!".into(),
            position: position.clone(),
        }
    }
    let checked_formula = match content[0] {
        Element::Text { ref text, .. } => {
            if settings.check_tex_formulas {
                texvccheck(text, settings)
            } else {
                let mut ret = "+".to_owned();
                ret.push_str(text);
                ret
            }
        },
        _ => return Element::Error {
            message: "A formula must only have text as content!".into(),
            position: position.clone(),
        }
    };
    let error_cause = match checked_formula.chars().next() {
        Some('+') => return Element::Text {
            position: position.clone(),
            text: checked_formula.replacen('+', "", 1),
        },
        Some('S') => "syntax error".into(),
        Some('E') => "lexing error".into(),
        Some('F') => format!("unknown function `{}`",
            checked_formula.chars().skip(1).collect::<String>()),
        Some('-') => "other error".into(),
        None => "empty string".into(),
        _ => "unknown error".into(),
    };
    Element::Error {
        message: error_cause,
        position: position.clone(),
    }
}

/// Call the external program `texvccheck` to check a Tex formula
fn texvccheck(formula: &str, settings: &Settings) -> String {
    let output = Command::new(&settings.texvccheck_path)
                         .arg(formula)
                         .output()
                         .expect("Failed to launch texvccheck!");
    String::from_utf8(output.stdout).expect("Corrupted texvccheck output!")
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
                let file_error = Element::Error {
                    position: position.to_owned(),
                    message: format!("section file `{}` could not \
                                be read or parsed!", &path)
                };

                let section_str = File::open(&path);

                if section_str.is_err() {
                    result.push(file_error);
                    return Ok(result);
                }

                let mut section_tree: Vec<Element>
                    = match serde_yaml::from_reader(&section_str.unwrap()) {
                    Ok(root) => root,
                    Err(_) => {
                        result.push(file_error);
                        return Ok(result);
                    }
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
        let template_name = extract_plain_text(name).trim().to_lowercase();
        if ["list", "liste"].contains(&template_name.as_str()) {

            let mut list_content = vec![];

            let list_type = if let Some(&Element::TemplateArgument {
                ref value,
                ..
            }) = find_arg(content, &["type".into()]) {
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
