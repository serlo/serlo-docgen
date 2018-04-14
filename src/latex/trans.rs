//! Implements formula text normalization for the `latex` target.

use mediawiki_parser::transformations::*;
use preamble::*;

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
            Element::InternalReference(ref iref) => {
                if is_thumb(iref) {
                    self.thumbs.push(root.clone());
                }
                Ok(false)
            },
            Element::Heading(_) => Ok(false),
            _ => Ok(true),
        }
    }
}

/// Move thumbnail images to a gallery under the current heading.
pub fn hoist_thumbnails(mut root: Element, settings: &Settings) -> TResult {
    if let Element::Heading(ref mut heading) = root {
        let thumbs = {
            let mut collector = ThumbCollector {
                path: vec![],
                thumbs: vec![],
            };
            collector.run_vec(&heading.content, settings, &mut vec![])
                .expect("error collecting thumbnails. HOW?");
            collector.thumbs
        };

        if !thumbs.is_empty() {
            let marker = TagAttribute {
                position: heading.position.clone(),
                key: "from_thumbs".into(),
                value: "true".into()
            };
            let gallery = Element::Gallery(Gallery {
                position: heading.position.clone(),
                attributes: vec![marker],
                content: thumbs,
            });
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
    settings: &'a Settings
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
