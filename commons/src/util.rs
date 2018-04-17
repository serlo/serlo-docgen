//! Common utilities for mfnf tools.

use mediawiki_parser::*;
use std::collections::HashMap;
use std::process::Command;

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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TexResult {
    Ok(String),
    UnknownFunction(String),
    LexingError,
    SyntaxError,
    UnknownError,
}

pub trait TexChecker {
    fn check(&mut self, source: &str) -> TexResult;
}

#[derive(Debug, PartialEq, Clone)]
pub struct CachedTexChecker {
    pub texvccheck_path: String,
    pub max_size: usize,
    pub cache: HashMap<String, TexResult>,
}

impl CachedTexChecker {
    pub fn new(path: &str, size: usize) -> CachedTexChecker {
        CachedTexChecker {
            texvccheck_path: path.into(),
            max_size: size,
            cache: HashMap::with_capacity(size),
        }
    }

    pub fn set_path(&mut self, path: &str) {
        self.texvccheck_path = path.into();
    }

    pub fn get_path(&self) -> &str {
        &self.texvccheck_path
    }
}

impl TexChecker for CachedTexChecker {
    fn check(&mut self, source: &str) -> TexResult {
        if let Some(result) = self.cache.get(source) {
            return result.clone()
        }

        let mut output = Command::new(&self.texvccheck_path).arg(source).output()
            .expect("Failed to launch texvccheck!");
        let mut iter = output.stdout.drain(..);
        let first = iter.next();
        let text = String::from_utf8(iter.collect())
            .expect("Corrupted texvccheck output!");
        let result = match first {
            Some(c) => match c as char {
                '+' => TexResult::Ok(text),
                'F' => TexResult::UnknownFunction(text),
                'S' => TexResult::SyntaxError,
                'E' => TexResult::LexingError,
                _ => TexResult::UnknownError,
            },
            _ => TexResult::UnknownError,
        };

        if self.cache.len() > self.max_size {
            let mut count = 0;
            self.cache.retain(|_, _| {
                count += 1;
                count % 10 != 1
            });
        }
        self.cache.insert(source.into(), result.clone());
        result
    }
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
