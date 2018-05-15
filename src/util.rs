//! Various utility functions and definitions.

use mediawiki_parser::*;
// re-export common util
pub use mwparser_utils::util::*;
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

/// Is a type just the default instance?
pub fn is_default<T>(obj: &T) -> bool where T: PartialEq + Default {
    return *obj == T::default();
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

/// Convert a filename to a make-friedly format.
pub fn filename_to_make(input: &str) -> String {
    input.replace(" ", "_")
        .replace(":", "@COLON@")
        .replace("(", "@LBR@")
        .replace(")", "@RBR@")
        .replace("/", "@SLASH@")
        .replace("'", "@SQUOTE@")
        .replace('"', "@DQUOTE@")
}

struct TreeMatcher<'e, 'c> {
    pub result: bool,
    pub path: Vec<&'e Element>,
    pub predicate: &'c Fn(&Element) -> bool,
}

impl<'e, 'c> Traversion<'e, ()> for TreeMatcher<'e, 'c> {
    path_methods!('e);

    fn work(&mut self, root: &Element, _: (), _: &mut io::Write) -> io::Result<bool> {
        if (self.predicate)(root) {
            self.result = true;
            Ok(false)
        } else {
            Ok(true)
        }
    }
}

/// recursively tests a predicate for a AST.
pub fn tree_contains(tree: &Element, predicate: &Fn(&Element) -> bool) -> bool {
    let mut matcher = TreeMatcher {
        result: false,
        path: vec![],
        predicate: predicate,
    };
    matcher.run(tree, (), &mut vec![])
        .expect("unexptected tree matcher IO error:");
    matcher.result
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
pub fn is_thumb(image: &InternalReference) -> bool {
    for option in &image.options {
        if extract_plain_text(option).to_lowercase().trim() == "thumb" {
            return true
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

    let section_file = filename_to_make(&settings.general.section_rev);
    let section_ext = &settings.general.section_ext;
    let section_path = &settings.general.section_path;
    let path = PathBuf::new()
        .join(&section_path)
        .join(&article)
        .join(&section)
        .join(&section_file)
        .with_extension(&section_ext);
    path.to_string_lossy().to_string()
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
        renderer.run(self, settings, &mut temp)?;
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
        renderer.run_vec(self, settings, &mut temp)?;
        Ok(String::from_utf8(temp).unwrap())
    }
}

/// Extract all child nodes from an elment in a list.
/// If an element has multiple fields, they are concatenated
/// in a semantically useful order.
pub fn extract_content(root: Element) -> Option<Vec<Element>> {
    match root {
        Element::Document(e) => Some(e.content),
        Element::Formatted(e) => Some(e.content),
        Element::Paragraph(e) => Some(e.content),
        Element::ListItem(e) => Some(e.content),
        Element::List(e) => Some(e.content),
        Element::TableCell(e) => Some(e.content),
        Element::HtmlTag(e) => Some(e.content),
        Element::Gallery(e) => Some(e.content),
        Element::Heading(mut e) => {
            e.caption.append(&mut e.content);
            Some(e.caption)
        },
        Element::Template(mut e) => {
            e.name.append(&mut e.content);
            Some(e.name)
        },
        Element::TemplateArgument(e) => Some(e.value),
        Element::InternalReference(mut e) => {
            for mut option in &mut e.options {
                e.target.append(&mut option);
            }
            e.target.append(&mut e.caption);
            Some(e.target)
        },
        Element::ExternalReference(e) => Some(e.caption),
        Element::Table(mut e) => {
            e.caption.append(&mut e.rows);
            Some(e.caption)
        }
        Element::TableRow(e) => Some(e.cells),
        Element::Text(_)
        | Element::Comment(_)
        | Element::Error(_)
        => None,
    }
}
