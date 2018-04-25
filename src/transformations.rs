use mediawiki_parser::transformations::*;
use mediawiki_parser::*;
use preamble::*;
use mfnf_commons::util::TexResult;
use std::fs::File;
use serde_yaml;


/// Convert template name paragraphs to lowercase text only.
pub fn normalize_template_names(mut root: Element, settings: &Settings) -> TResult {
    if let Element::Template(ref mut template) = root {

        if template.name.is_empty() {
            return Ok(Element::Error(Error {
                position: template.position.clone(),
                message: "MFNF template name must not be empty!".to_string(),
            }))
        };

        let mut new_text = extract_plain_text(&template.name).trim().to_owned();

        for child in &mut template.content {
            if let Element::TemplateArgument(ref mut arg) = *child {
                let lowercase = arg.name.trim().to_lowercase();
                arg.name.clear();
                arg.name.push_str(&lowercase);
            } else {
                return Ok(Element::Error(Error {
                    position: template.position.clone(),
                    message: "Only TemplateArguments are allowed as \
                            children of templates!".into(),
                }))
            }
        }

        if !new_text.is_empty() {

            // convert to lowercase and remove prefixes
            if !new_text.starts_with('#') {
                new_text = new_text.trim().to_lowercase();
            }

            let text = Element::Text(Text {
                position: Span {
                    start: template.name.first()
                        .map(|e| e.get_position().start.clone())
                        .unwrap_or(template.position.start.clone()),
                    end: template.name.last()
                        .map(|e| e.get_position().end.clone())
                        .unwrap_or(template.position.end.clone()),
                },
                text: new_text,
            });
            template.name.clear();
            template.name.push(text);
        } else {
            return Ok(Element::Error(Error {
                position: template.position.clone(),
                message: "Template names cannot be empty!".into(),
            }));
        }
    };
    recurse_inplace(&normalize_template_names, root, settings)
}


/// Normalize math formulas with texvccheck
pub fn normalize_math_formulas(mut root: Element, settings: &Settings) -> TResult {

    if let Element::Formatted(ref mut formatted) = root {
        if formatted.markup == MarkupType::Math {
            match check_formula(&formatted.content, &formatted.position, settings) {
                e @ Element::Text(_) => {
                    formatted.content.clear();
                    formatted.content.push(e);
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
        return Element::Error(Error {
            message: "A formula must have exactly one content element!".into(),
            position: position.clone(),
        })
    }
    let checked_formula = match content[0] {
        Element::Text(ref text) => {
            if let Some(ref mutex) = settings.tex_checker {
                mutex.check(&text.text)
            } else {
                return content[0].clone();
            }
        },
        _ => return Element::Error(Error {
            message: "A formula must only have text as content!".into(),
            position: position.clone(),
        })
    };
    let cause = match checked_formula {
        TexResult::Ok(content) => {
            return Element::Text(Text {
                position: position.clone(),
                text: content,
            });
        },
        TexResult::UnknownFunction(func) => format!("unknown latex function `{}`!", func),
        TexResult::SyntaxError => "latex syntax error!".into(),
        TexResult::LexingError => "latex lexer error!".into(),
        TexResult::UnknownError => "unknown latex error!".into(),
    };

    Element::Error(Error {
        message: cause,
        position: position.clone(),
    })
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

        if let Element::Template(ref template) = child {
            let prefix = &settings.section_inclusion_prefix;
            let template_name = extract_plain_text(&template.name);

            // section transclusion
            if template_name.to_lowercase().trim().starts_with(prefix) {
                let article = trim_prefix(template_name.trim(), prefix);
                if template.content.len() < 1 {
                    return Err(TransformationError {
                        cause: "A section inclusion must specify article \
                                name and section name!".to_string(),
                        position: template.position.clone(),
                        transformation_name: "include_sections".to_string(),
                        tree: Element::Template(template.clone())
                    });
                }

                let section_name = extract_plain_text(&template.content);
                let path = get_section_path(article, &section_name, settings);

                // error returned when the section file is faulty
                let file_error = Element::Error(Error {
                    position: template.position.clone(),
                    message: format!("section file `{}` could not \
                                be read or parsed!", &path)
                });

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
                    Element::Comment(Comment {
                        position: template.position.clone(),
                        text: format!("included from: {}|{}", article, section_name),
                    })
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

    if let Element::Heading(ref mut heading) = root {
        heading.depth = current_depth;
        current_depth += 1;
    }

    recurse_inplace(&normalize_heading_depths_traverse, root, current_depth)
}

/// Remove prefixes before filenames of included files.
pub fn remove_file_prefix(mut root: Element, settings: &Settings) -> TResult {
    if let Element::InternalReference(ref mut iref) = root {
        if let Some(&mut Element::Text(ref mut text)) = iref.target.first_mut() {
            for prefix in &settings.file_prefixes {
                let prefix_str = format!("{}:", &prefix);
                let new_text = String::from(trim_prefix(&text.text, &prefix_str));
                text.text.clear();
                text.text.push_str(&new_text);
            }
        }
    }
    recurse_inplace(&remove_file_prefix, root, settings)
}
