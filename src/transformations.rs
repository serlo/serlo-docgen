use mediawiki_parser::transformations::*;
use mediawiki_parser::*;
use mfnf_sitemap::Subtarget;
use preamble::*;
use serde_yaml;
use std::fs::File;

/// Convert template name paragraphs to lowercase text only.
pub fn normalize_template_names(mut root: Element, settings: &Settings) -> TResult {
    if let Element::Template(ref mut template) = root {
        if template.name.is_empty() {
            return Ok(Element::Error(Error {
                position: template.position.clone(),
                message: "MFNF template name must not be empty!".to_string(),
            }));
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
                              children of templates!"
                        .into(),
                }));
            }
        }

        if !new_text.is_empty() {
            // convert to lowercase and remove prefixes
            if !new_text.starts_with('#') {
                new_text = new_text.trim().to_lowercase();
            }

            let text = Element::Text(Text {
                position: Span {
                    start: template
                        .name
                        .first()
                        .map(|e| e.get_position().start.clone())
                        .unwrap_or_else(|| template.position.start.clone()),
                    end: template
                        .name
                        .last()
                        .map(|e| e.get_position().end.clone())
                        .unwrap_or_else(|| template.position.end.clone()),
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

pub fn include_sections(root: Element, settings: &Settings) -> TResult {
    recurse_inplace_template(&include_sections, root, settings, &include_sections_vec)
}

pub fn include_sections_vec<'a>(
    trans: &TFuncInplace<&'a Settings>,
    root_content: &mut Vec<Element>,
    settings: &'a Settings,
) -> TListResult {
    // search for section inclusion in children
    let mut result = vec![];
    for mut child in root_content.drain(..) {
        if let Element::Template(ref template) = child {
            let prefix = &settings.general.section_inclusion_prefix;
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
                let path = get_section_path(article, &section_name, settings);

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
                    match serde_yaml::from_reader(&section_str.unwrap()) {
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
                    include_sections_vec(&include_sections, &mut section_tree, settings)?;
                result.append(&mut section_tree);
                continue;
            }
        }
        result.push(trans(child, settings)?);
    }
    Ok(result)
}

/// Normalize heading depths by making subheadings one level deeper than their parent.
/// The highest level of headings is assigned depth 1.
pub fn normalize_heading_depths(root: Element, _settings: &Settings) -> TResult {
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

fn remove_exclusions_vec<'a>(
    trans: &TFuncInplace<&'a Settings>,
    root_content: &mut Vec<Element>,
    settings: &'a Settings,
) -> TListResult {
    let mut result = vec![];
    let (subtarget, include) = {
        let is_current_subtarget = |s: &&Subtarget| -> bool {
            s.name == settings.runtime.target_name.trim().to_lowercase()
        };

        let include_subtarget = settings
            .runtime
            .markers
            .include
            .subtargets
            .iter()
            .find(&is_current_subtarget);
        let exclude_subtarget = settings
            .runtime
            .markers
            .exclude
            .subtargets
            .iter()
            .find(&is_current_subtarget);

        if let Some(subtarget) = include_subtarget {
            (subtarget, true)
        } else if let Some(subtarget) = exclude_subtarget {
            (subtarget, false)
        } else {
            result.append(root_content);
            return Ok(result);
        }
    };

    if subtarget.parameters.is_empty() {
        result.append(root_content);
        return Ok(result);
    }

    for elem in root_content.drain(..) {
        if let Element::Heading(heading) = elem {
            let caption = extract_plain_text(&heading.caption).trim().to_lowercase();
            let in_params = subtarget
                .parameters
                .iter()
                .any(|h| h.trim().to_lowercase() == caption);

            let is_heading = |e: &Element| {
                if let Element::Heading(_) = e {
                    true
                } else {
                    false
                }
            };
            let new_heading = Element::Heading(heading);

            // if heading is not in list, inclusion depends on children
            if !in_params {
                let new_heading = trans(new_heading, settings)?;
                let contains_headings = if let Element::Heading(ref h) = new_heading {
                    h.content.iter().any(|e| tree_contains(e, &is_heading))
                } else {
                    unreachable!();
                };
                if !include || contains_headings {
                    result.push(new_heading)
                }
            // otherwise, only include heading when marked as include.
            } else if include {
                result.push(new_heading);
            }
        } else {
            result.push(trans(elem, settings)?);
        }
    }

    Ok(result)
}

fn check_heading_existence(
    root: &Element,
    subtarget: &Subtarget,
) -> Result<(), TransformationError> {
    for title in &subtarget.parameters {
        let matches = |e: &Element| {
            if let Element::Heading(ref h) = e {
                let caption = extract_plain_text(&h.caption).trim().to_lowercase();
                if title.trim().to_lowercase() == caption {
                    return true;
                }
            }
            false
        };
        if !tree_contains(root, &matches) {
            return Err(TransformationError {
                cause: format!(
                    "heading \"{}\" in \"{}\" is not present in this document!",
                    &title, &subtarget.name
                ),
                position: root.get_position().clone(),
                transformation_name: "remove_exclusions".to_string(),
                tree: Element::Error(Error {
                    position: root.get_position().clone(),
                    message: "heading not found".into(),
                }),
            });
        }
    }
    Ok(())
}

/// Strip excluded headings.
pub fn remove_exclusions(root: Element, settings: &Settings) -> TResult {
    // check if every specified heading exists
    if let Element::Document(_) = root {
        for subtarget in &settings.runtime.markers.include.subtargets {
            check_heading_existence(&root, &subtarget)?;
        }
        for subtarget in &settings.runtime.markers.exclude.subtargets {
            check_heading_existence(&root, &subtarget)?;
        }
    }
    recurse_inplace_template(&remove_exclusions, root, settings, &remove_exclusions_vec)
}

/// Resolve interwiki links.
pub fn resolve_interwiki_links(root: Element, settings: &Settings) -> TResult {
    if let Element::InternalReference(ref iref) = root {
        let text = extract_plain_text(&iref.target);
        if let Some(position) = text.find(":") {
            let interlink_result = settings
                .general
                .interwiki_link_mapping
                .get(text[0..position + 1].to_lowercase().trim());

            if let Some(replacement) = interlink_result {
                let reference = ExternalReference {
                    position: iref.position.clone(),
                    target: {
                        let mut r = replacement.clone();
                        r.push_str(&text[position + 1..]);
                        r
                    },
                    caption: iref.caption.clone(),
                };
                return Ok(Element::ExternalReference(reference));
            }
        }
    }
    recurse_inplace(&resolve_interwiki_links, root, settings)
}

/// Strip trailing whitespace elements from containers.
pub fn remove_whitespace_trailers(mut root: Element, settings: &Settings) -> TResult {
    fn rstrip<'a>(root_content: &mut Vec<Element>) {
        loop {
            let last = root_content.pop();
            if let Some(Element::Text(ref text)) = last {
                if text.text.trim().is_empty() {
                    continue;
                }
            }
            if let Some(last) = last {
                root_content.push(last);
            }
            break;
        }
    }

    match root {
        Element::Paragraph(ref mut par) => rstrip(&mut par.content),
        Element::TemplateArgument(ref mut arg) => rstrip(&mut arg.value),
        Element::InternalReference(ref mut iref) => rstrip(&mut iref.caption),
        Element::ExternalReference(ref mut eref) => rstrip(&mut eref.caption),
        Element::ListItem(ref mut li) => rstrip(&mut li.content),
        Element::TableCell(ref mut tc) => rstrip(&mut tc.content),
        _ => (),
    }
    recurse_inplace(&remove_whitespace_trailers, root, settings)
}

/// Unpack the paragraph in template arguments if they contain one paragraph
/// as their only content element. Usually, the user wanted no paragraph here.
pub fn unpack_template_arguments(mut root: Element, settings: &Settings) -> TResult {
    if let Element::TemplateArgument(ref mut arg) = root {
        let mut new_content = None;
        if let [Element::Paragraph(ref mut par)] = arg.value[..] {
            new_content = Some(par.content.drain(..).collect());
        }
        if let Some(mut new) = new_content {
            arg.value.clear();
            arg.value.append(&mut new);
        }
    }
    recurse_inplace(&unpack_template_arguments, root, settings)
}

/// Collects all thumbnail images on the current hierarchy layer.
pub struct ThumbCollector<'e> {
    pub path: Vec<&'e Element>,
    pub thumbs: Vec<Element>,
}

impl<'e, 's: 'e> Traversion<'e, &'s Settings> for ThumbCollector<'e> {
    path_methods!('e);

    fn work(&mut self, root: &'e Element, _: &'s Settings, _: &mut io::Write) -> io::Result<bool> {
        match *root {
            Element::InternalReference(ref iref) => {
                if is_thumb(iref) {
                    self.thumbs.push(root.clone());
                }
                Ok(false)
            }
            Element::Heading(_) => Ok(false),
            _ => Ok(true),
        }
    }
}

/// Move thumbnail images to a gallery under the current heading.
pub fn hoist_thumbnails(mut root: Element, settings: &Settings) -> TResult {
    if let Element::Heading(ref mut heading) = root {
        let mut thumbs = {
            let mut collector = ThumbCollector {
                path: vec![],
                thumbs: vec![],
            };
            collector
                .run_vec(&heading.content, settings, &mut vec![])
                .expect("error collecting thumbnails. HOW?");
            collector.thumbs
        };

        if !thumbs.is_empty() {
            // single thumb
            let gallery = if thumbs.len() == 1 {
                let mut img = thumbs.pop().unwrap();
                if let Element::InternalReference(ref mut iref) = img {
                    iref.options.clear();
                    iref.options.push(vec![Element::Text(Text {
                        position: Span::any(),
                        text: "center".into(),
                    })]);
                    iref.options.push(vec![Element::Text(Text {
                        position: Span::any(),
                        text: "from_thumb".into(),
                    })]);
                }
                img
            // multiple thumbs
            } else {
                let marker = TagAttribute {
                    position: heading.position.clone(),
                    key: "from_thumbs".into(),
                    value: "true".into(),
                };
                Element::Gallery(Gallery {
                    position: heading.position.clone(),
                    attributes: vec![marker],
                    content: thumbs,
                })
            };
            heading.content.insert(0, gallery);
        }
    };
    if let Element::Gallery(_) = root {
        Ok(root)
    } else {
        recurse_inplace_template(&hoist_thumbnails, root, settings, &hoist_thumbnails_vec)
    }
}

/// Delete thumnail thumnail images.
fn hoist_thumbnails_vec<'a>(
    trans: &TFuncInplace<&'a Settings>,
    root_content: &mut Vec<Element>,
    settings: &'a Settings,
) -> TListResult {
    let mut result = vec![];
    for mut child in root_content.drain(..) {
        if let Element::InternalReference(iref) = child {
            if !is_thumb(&iref) {
                result.push(trans(Element::InternalReference(iref), settings)?);
            }
        } else {
            result.push(trans(child, settings)?);
        }
    }
    Ok(result)
}
