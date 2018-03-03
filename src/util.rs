//! Various utility functions and definitions.

use mediawiki_parser::*;
use std::path::{PathBuf};
use std::process;
use settings::Settings;
use std::io;


/// Escape LaTeX-Specific symbols
pub fn escape_latex(input: &str) -> String {
    let mut res = String::new();
    for c in input.chars() {
        let s = match c {
            '$' => "\\$",
            '%' => "\\%",
            '&' => "\\&",
            '#' => "\\#",
            '_' => "\\_",
            '{' => "\\{",
            '}' => "\\}",
            '[' => "{[}",
            ']' => "{]}",
            '\"' => "{''}",
            '\\' => "\\textbackslash{}",
            '~' => "\\textasciitilde{}",
            '<' => "\\textless{}",
            '>' => "\\textgreater{}",
            '^' => "\\textasciicircum{}",
            '`' => "{}`",   // avoid ?` and !`
            '\n' => "\\\\",
            '↯' => "\\Lightning{}",
            _ => {
                res.push(c);
                continue
            },
        };
        res.push_str(s);
    }
    res
}

/// Trim one pair of prefix and suffix from a string, ignoring input case.
pub fn trim_enclosing<'a>(input: &'a str, prefix: &str, suffix: &str) -> &'a str {
    let lower_input = input.to_lowercase();
    if lower_input.starts_with(prefix) && lower_input.ends_with(suffix) {
        return &input[prefix.len()..input.len()-suffix.len()];
    }
    input
}

/// Remove a prefix if found, ignoring input case.
pub fn trim_prefix<'a>(input: &'a str, prefix: &str) -> &'a str {
    let lower_input = input.to_lowercase();
    if lower_input.starts_with(prefix) {
        return &input[prefix.len()..];
    }
    input
}

/// Indent and trim a string.
pub fn indent_and_trim(input: &str, depth: usize, max_line_width: usize) -> String {
    const COMMENT_PREFIX: &str = "% ";

    let mut lines = vec![];
    for line in input.split('\n') {
        let trimmed = line.trim();
        let comment = trimmed.starts_with(COMMENT_PREFIX.trim());
        let line_depth = depth + line.len() - line.trim_left().len();
        let start_string = format!("{:depth$}", "", depth=line_depth);

        let mut new_line = start_string.clone();

        if trimmed.len() > max_line_width {

            for word in trimmed.split(' ') {

                let current_length = new_line.trim().len();

                if current_length + word.len() + 1 > max_line_width && current_length > 0 {
                    lines.push(new_line);
                    new_line = start_string.clone();
                    if comment {
                        new_line.push_str(COMMENT_PREFIX);
                    }
                }

                new_line.push_str(word);
                new_line.push(' ');
            }
            lines.push(new_line);
        } else {
            new_line.push_str(trimmed);
            lines.push(new_line);
        }
    }
    lines.join("\n")
}

/// Returns the template argument with a given name from a list.
pub fn find_arg<'a>(content: &'a [Element], arg_name: &str) -> Option<&'a Element> {
    for child in content {
        if let Element::TemplateArgument { ref name, .. } = *child {
            if name == arg_name {
                return Some(child);
            }
        }
    }
    None
}


/// Extract plain text (Paragraph and Text nodes) from a list of nodes and concatenate it.
pub fn extract_plain_text(content: &[Element]) -> String {
    let mut result = String::new();
    for root in content {
        match *root {
            Element::Text { ref text, .. } => {
                result.push_str(text);
            },
            Element::Paragraph { ref content, .. } => {
                result.push_str(&extract_plain_text(content));
            },
            Element::TemplateArgument { ref value, .. } => {
                result.push_str(&extract_plain_text(value));
            },
            _ => (),
        };
    }
    result
}

/// Convert a filename to a make-friedly format.
pub fn filename_to_make(input: &str) -> String {
    input.replace(" ", "_")
        .replace(":", "@COLON@")
        .replace("(", "@LBR@")
        .replace(")", "@RBR@")
}

/// verifies a given "path" is only a plain filename without directory structure.
fn is_plain_file(path: &PathBuf) -> bool {
    let components = path.components();
    if components.count() != 1 {
        return false
    }
    match path.components().next() {
        Some(c) => c.as_os_str() == path,
        None => false
    }
}

/// Returns wether an image is semantically a thumbnail image.
pub fn is_thumb(image: &Element) -> bool {
    if let Element::InternalReference {
        ref options,
        ..
    } = *image {
        for option in options {
            if extract_plain_text(option).to_lowercase().trim() == "thumb" {
                return true
            }
        }
    }
    false
}

/// Path of a section file.
pub fn get_section_path(article: &str, section: &str, settings: &Settings) -> String {
    if !is_plain_file(&PathBuf::from(article)) {
        eprintln!("article name \"{}\" contains path elements. \
                   This could be dangerous! Abort.", article);
        process::exit(1);
    }

    if !is_plain_file(&PathBuf::from(section)) {
        eprintln!("section name \"{}\" contains path elements. \
                   This could be dangerous! Abort.", section);
        process::exit(1);
    }

    let section_file = &settings.section_rev;
    let section_ext = &settings.section_ext;
    let section_path = &settings.section_path;
    let path = PathBuf::new()
        .join(&section_path)
        .join(&article)
        .join(&section)
        .join(&section_file)
        .with_extension(&section_ext);
    filename_to_make(&path.to_string_lossy())
}

/// generates getters and setters for a path member of a traversion.
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

/// This object can be rendered by a traversion.
pub trait Renderable<S>  {
    fn render<'e, 's>(
        &'e self,
        renderer: &mut Traversion<'e, &'s S>,
        settings: &'s S,
    ) -> io::Result<String>;
}

impl<S> Renderable<S> for Element {
    fn render<'e, 's>(
        &'e self,
        renderer: &mut Traversion<'e, &'s S>,
        settings: &'s S,
    ) -> io::Result<String> {

        let mut temp = vec![];
        renderer.run(&self, settings, &mut temp)?;
        Ok(String::from_utf8(temp).unwrap())
    }
}

impl<S> Renderable<S> for [Element] {
    fn render<'e, 's>(
        &'e self,
        renderer: &mut Traversion<'e, &'s S>,
        settings: &'s S,
    ) -> io::Result<String> {

        let mut temp = vec![];
        renderer.run_vec(&self, settings, &mut temp)?;
        Ok(String::from_utf8(temp).unwrap())
    }
}

/// Extract all child nodes from an elment in a list.
/// If an element has multiple fields, they are concatenated
/// in a semantically useful order.
pub fn extract_content(root: Element) -> Option<Vec<Element>> {
    match root {
        Element::Document { content, .. }
        | Element::Formatted { content, .. }
        | Element::Paragraph { content, .. }
        | Element::ListItem { content, .. }
        | Element::List { content, .. }
        | Element::TableCell { content, .. }
        | Element::HtmlTag { content, .. }
        | Element::Gallery { content, .. }
        => Some(content),
        Element::Heading { mut caption, mut content, .. } => {
            caption.append(&mut content);
            Some(caption)
        },
        Element::Template { mut name, mut content, .. } => {
            name.append(&mut content);
            Some(name)
        },
        Element::TemplateArgument { value, .. } => Some(value),
        Element::InternalReference { mut target, options, mut caption, .. } => {
            for mut option in options {
                target.append(&mut option);
            }
            target.append(&mut caption);
            Some(target)
        },
        Element::ExternalReference { caption, .. } => Some(caption),
        Element::Table { mut caption, mut rows, .. } => {
            caption.append(&mut rows);
            Some(caption)
        }
        Element::TableRow { cells, .. } => Some(cells),
        Element::Text { .. }
        | Element::Comment { .. }
        | Element::Error { .. }
        => None,
    }
}
