use mediawiki_parser::transformations::*;
use mediawiki_parser::*;
use preamble::*;

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

/// Resolve interwiki links.
pub fn resolve_interwiki_links(root: Element, settings: &Settings) -> TResult {
    if let Element::InternalReference(ref iref) = root {
        let text = extract_plain_text(&iref.target);
        if let Some(position) = text.find(':') {
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

/// Delete empty template arguments.
pub fn remove_empty_arguments(mut root: Element, settings: &Settings) -> TResult {
    // check if every specified heading exists
    if let Element::Template(ref mut template) = root {
        let new_content = template
            .content
            .drain(..)
            .filter(|arg| {
                if let Element::TemplateArgument(arg) = arg {
                    // value must not contain any non-whitespace elements
                    arg.value.iter().any(|ref elem| {
                        tree_contains(&elem, &|ref elem| match elem {
                            Element::Paragraph(_) => false,
                            Element::Text(text) => !text.text.trim().is_empty(),
                            _ => true,
                        })
                    })
                } else {
                    true
                }
            }).collect();
        template.content = new_content;
    }
    recurse_inplace(&remove_empty_arguments, root, settings)
}
