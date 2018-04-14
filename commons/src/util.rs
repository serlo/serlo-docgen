//! Common utilities for mfnf tools.

use mediawiki_parser::*;

/// generates getters and setters for a path member of a traversion.
#[macro_export]
macro_rules! path_methods {
    ($lt:tt) => {
        fn path_push(&mut self, root: &$lt Element) {
            self.path.push(root);
        }
        fn path_pop(&mut self) -> Option<&$lt Element> {
            self.path.pop()
        }
        fn get_path(&self) -> &Vec<&$lt Element> {
            &self.path
        }
    }
}

/// Extract plain text (Paragraph and Text nodes) from a list of nodes and concatenate it.
pub fn extract_plain_text(content: &[Element]) -> String {
    let mut result = String::new();
    for root in content {
        match *root {
            Element::Text(ref e) => {
                result.push_str(&e.text);
            },
            Element::Formatted(ref e) => {
                result.push_str(&extract_plain_text(&e.content));
            },
            Element::Paragraph(ref e) => {
                result.push_str(&extract_plain_text(&e.content));
            },
            Element::TemplateArgument(ref e) => {
                result.push_str(&extract_plain_text(&e.value));
            },
            _ => (),
        };
    }
    result
}

/// Returns the template argument with a matching name (lowercase) from a list.
pub fn find_arg<'a>(content: &'a [Element], names: &[String]) -> Option<&'a Element> {
    for child in content {
        if let Element::TemplateArgument(ref e) = *child {
            if names.contains(&e.name.trim().to_lowercase()) {
                return Some(child);
            }
        }
    }
    None
}
