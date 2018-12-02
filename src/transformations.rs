use mediawiki_parser::transformations::*;
use mediawiki_parser::*;
use mfnf_sitemap::{Markers, Subtarget};
use preamble::*;

fn remove_exclusions_vec<'a>(
    trans: &TFuncInplace<&'a Markers>,
    root_content: &mut Vec<Element>,
    markers: &'a Markers,
) -> TListResult {
    let mut result = vec![];
    let (subtarget, include) = {
        let include_subtarget = markers
            .include
            .subtargets
            .iter()
            .find(|s| &s.name == "current");
        let exclude_subtarget = markers
            .exclude
            .subtargets
            .iter()
            .find(|s| &s.name == "current");

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
                let new_heading = trans(new_heading, markers)?;
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
            result.push(trans(elem, markers)?);
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
pub fn remove_exclusions(root: Element, markers: &Markers) -> TResult {
    // check if every specified heading exists
    if let Element::Document(_) = root {
        for subtarget in &markers.include.subtargets {
            check_heading_existence(&root, &subtarget)?;
        }
        for subtarget in &markers.exclude.subtargets {
            check_heading_existence(&root, &subtarget)?;
        }
    }
    recurse_inplace_template(&remove_exclusions, root, markers, &remove_exclusions_vec)
}

/// Collects all thumbnail images on the current hierarchy layer.
pub struct ThumbCollector<'e> {
    pub path: Vec<&'e Element>,
    pub thumbs: Vec<Element>,
}

impl<'e, 's: 'e> Traversion<'e, &'s ()> for ThumbCollector<'e> {
    path_methods!('e);

    fn work(&mut self, root: &'e Element, _: &'s (), _: &mut io::Write) -> io::Result<bool> {
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
pub fn hoist_thumbnails(mut root: Element, _: ()) -> TResult {
    if let Element::Heading(ref mut heading) = root {
        let mut thumbs = {
            let mut collector = ThumbCollector {
                path: vec![],
                thumbs: vec![],
            };
            collector
                .run_vec(&heading.content, &(), &mut vec![])
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
        recurse_inplace_template(&hoist_thumbnails, root, (), &hoist_thumbnails_vec)
    }
}

/// Delete thumnail thumnail images.
fn hoist_thumbnails_vec(
    trans: &TFuncInplace<()>,
    root_content: &mut Vec<Element>,
    _: (),
) -> TListResult {
    let mut result = vec![];
    for mut child in root_content.drain(..) {
        if let Element::InternalReference(iref) = child {
            if !is_thumb(&iref) {
                result.push(trans(Element::InternalReference(iref), ())?);
            }
        } else {
            result.push(trans(child, ())?);
        }
    }
    Ok(result)
}
