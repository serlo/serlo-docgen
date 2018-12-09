use crate::preamble::*;
use mediawiki_parser::transformations::*;
use mediawiki_parser::*;
use std::fs::File;
use std::path::PathBuf;

pub fn include_sections(root: Element, section_path: &PathBuf) -> TResult {
    recurse_inplace_template(&include_sections, root, section_path, &include_sections_vec)
}

pub fn include_sections_vec<'a>(
    trans: &TFuncInplace<&'a PathBuf>,
    root_content: &mut Vec<Element>,
    section_path: &'a PathBuf,
) -> TListResult {
    // search for section inclusion in children
    let mut result = vec![];
    for child in root_content.drain(..) {
        if let Element::Template(ref template) = child {
            let prefix = SECTION_INCLUSION_PREFIX;
            let template_name = extract_plain_text(&template.name);

            // section transclusion
            if template_name.to_lowercase().trim().starts_with(prefix) {
                let article = trim_prefix(template_name.trim(), prefix);
                if template.content.is_empty() {
                    return Err(TransformationError {
                        cause: "A section inclusion must specify article \
                                name and section name!"
                            .to_string(),
                        position: template.position.clone(),
                        transformation_name: "include_sections".to_string(),
                        tree: Element::Template(template.clone()),
                    });
                }

                let section_name = extract_plain_text(&template.content);
                let path = get_section_path(article, &section_name, section_path);

                // error returned when the section file is faulty
                let file_error = Element::Error(Error {
                    position: template.position.clone(),
                    message: format!(
                        "section file `{}` could not \
                         be read or parsed!",
                        &path
                    ),
                });

                let section_str = File::open(&path);

                if section_str.is_err() {
                    result.push(file_error);
                    return Ok(result);
                }

                let mut section_tree: Vec<Element> =
                    match serde_json::from_reader(&section_str.unwrap()) {
                        Ok(root) => root,
                        Err(_) => {
                            result.push(file_error);
                            return Ok(result);
                        }
                    };

                result.push(Element::Comment(Comment {
                    position: template.position.clone(),
                    text: format!("included from: {}|{}", article, section_name),
                }));

                // recursively include sections
                // heading depths are normalized in a later transformation
                section_tree =
                    include_sections_vec(&include_sections, &mut section_tree, section_path)?;
                result.append(&mut section_tree);
                continue;
            }
        }
        result.push(trans(child, section_path)?);
    }
    Ok(result)
}

/// Normalize heading depths by making subheadings one level deeper than their parent.
/// The highest level of headings is assigned depth 1.
pub fn normalize_heading_depths(root: Element, _: ()) -> TResult {
    normalize_heading_depths_traverse(root, 1)
}

fn normalize_heading_depths_traverse(mut root: Element, current_depth: usize) -> TResult {
    let mut current_depth = current_depth;

    if let Element::Heading(ref mut heading) = root {
        heading.depth = current_depth;
        current_depth += 1;
    }

    recurse_inplace(&normalize_heading_depths_traverse, root, current_depth)
}
