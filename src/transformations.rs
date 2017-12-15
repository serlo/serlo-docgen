use mediawiki_parser::ast::*;
use mediawiki_parser::transformations::*;
use settings::Settings;


/// Convert template name paragraphs to lowercase text only.
pub fn normalize_template_names(mut root: Element, settings: &Settings) -> TResult {
    match &mut root {
        &mut Element::Template { ref mut name, ref position, .. } => {

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

            match new_text.first() {
                Some(&Element::Text { ref position, ref text }) => {

                    name.clear();
                    name.push(
                        Element::Text {
                            position: position.clone(),
                            text: text.clone().to_lowercase(),
                        }
                    );
                },
                _ => {
                    return Ok(Element::Error {
                        position: match new_text.first() {
                            Some(e) => { e.get_position().clone() },
                            None => { position.clone() },
                        },
                        message: "MFNF Template names must be plain strings \
                                With no markup and are case insensitive!".to_string(),
                    });
                }
            }

        },
        _ => (),
    };
    recurse_inplace(&normalize_template_names, root, settings)
}
