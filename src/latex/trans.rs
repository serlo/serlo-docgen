//! Implements formula text normalization for the `latex` target.

use mediawiki_parser::transformations::*;
use mediawiki_parser::{MarkupType, TagAttribute};
use preamble::*;

/// Transform a formula template argument to text-only.
pub fn normalize_formula(mut root: Element, settings: &Settings) -> TResult {

    if let Element::Template {
        ref name,
        ref mut content,
        ref position,
        ..
    } = root {

        let template_name = extract_plain_text(name);
        if &template_name == "formula" {
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

                    let is_math = if let MarkupType::Math = *markup {true} else {false};
                    if content.len() != 1 || !is_math {
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

/// Collects all thumbnail images on the current hierarchy layer.
pub struct ThumbCollector<'e> {
    pub path: Vec<&'e Element>,
    pub thumbs: Vec<Element>,
}

impl<'e, 's: 'e> Traversion<'e, &'s Settings> for ThumbCollector<'e> {

    path_methods!('e);

    fn work(
        &mut self,
        root: &'e Element,
        _: &'s Settings,
        _: &mut io::Write
    ) -> io::Result<bool> {
        match *root {
            Element::InternalReference { .. } => {
                if is_thumb(root) {
                    self.thumbs.push(root.clone());
                }
                Ok(false)
            },
            Element::Heading { .. } => Ok(false),
            _ => Ok(true),
        }
    }
}

/// Move thumbnail images to a gallery under the current heading.
pub fn hoist_thumbnails(mut root: Element, settings: &Settings) -> TResult {
    if let Element::Heading {
        ref position,
        ref mut content,
        ..
    } = root {
        let thumbs = {
            let mut collector = ThumbCollector {
                path: vec![],
                thumbs: vec![],
            };
            collector.run_vec(content, settings, &mut vec![])
                .expect("error collecting thumbnails. HOW?");
            collector.thumbs
        };

        if !thumbs.is_empty() {
            let marker = TagAttribute {
                position: position.clone(),
                key: "from_thumbs".into(),
                value: "true".into()
            };
            let gallery = Element::Gallery {
                position: position.clone(),
                attributes: vec![marker],
                content: thumbs,
            };
            content.insert(0, gallery);
        }
    };
    if let Element::Gallery { .. } = root {
        Ok(root)
    } else {
        recurse_inplace_template(&hoist_thumbnails, root, settings, &hoist_thumbnails_vec)
    }
}

/// Delete thumnail thumnail images.
fn hoist_thumbnails_vec<'a>(
    trans: &TFuncInplace<&'a Settings>,
    root_content: &mut Vec<Element>,
    settings: &'a Settings
) -> TListResult {
    let mut result = vec![];
    for mut child in root_content.drain(..) {
        if let Element::InternalReference { .. } = child {
            if !is_thumb(&child) {
                result.push(trans(child, settings)?);
            }
        } else {
            result.push(trans(child, settings)?);
        }
    }
    Ok(result)
}
