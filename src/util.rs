//! Various utility functions and definitions.

use mediawiki_parser::*;
use std::path::PathBuf;
use settings::Settings;


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
pub fn indent_and_trim<'a>(input: &'a str, depth: usize, max_line_width: usize) -> String {
    const COMMENT_PREFIX: &str = "% ";

    let mut lines = vec![];
    for line in input.split("\n") {
        let comment = line.trim().starts_with(COMMENT_PREFIX.trim());
        let start_string = format!("{:depth$}", "", depth=depth);
        let mut current_line = start_string.clone();

        if line.trim().len() > max_line_width && line.trim().matches(" ").count() > 0 {

            for word in line.split(" ") {
                if current_line.trim().len() + word.len() + 1 > max_line_width {
                    lines.push(current_line);
                    current_line = start_string.clone();
                    if comment {
                        current_line.push_str(COMMENT_PREFIX);
                    }
                }

                current_line.push_str(word);
                current_line.push_str(" ");
            }
            lines.push(current_line);
        } else {
            current_line.push_str(line);
            lines.push(current_line);
        }
    }
    lines.join("\n")
}

/// Returns the template argument with a given name from a list.
pub fn find_arg<'a>(content: &'a Vec<Element>, arg_name: &str) -> Option<&'a Element> {
    for child in content {
        if let &Element::TemplateArgument { ref name, .. } = child {
            if name == arg_name {
                return Some(child);
            }
        }
    }
    None
}


/// Extract plain text (Paragraph and Text nodes) from a list of nodes and concatenate it.
pub fn extract_plain_text(content: &Vec<Element>) -> String {
    let mut result = String::new();
    for root in content {
        match root {
            &Element::Text { ref text, .. } => {
                result.push_str(text);
            },
            &Element::Paragraph { ref content, .. } => {
                result.push_str(&extract_plain_text(content));
            },
            &Element::TemplateArgument { ref value, .. } => {
                result.push_str(&extract_plain_text(value));
            },
            _ => (),
        };
    }
    result
}

/// Convert a filename to a make-friedly format.
pub fn filename_to_make(input: &str) -> String {
    input.replace(" ", "_").replace(":", "@COLON@")
}

/// Path of a section file.
pub fn get_section_path(article: &str, section: &str, settings: &Settings) -> String {
    let section_file = &settings.section_rev;
    let section_ext = &settings.section_ext;
    let section_path = &settings.section_path;
    let path = PathBuf::new()
        .join(&section_path)
        .join(&article)
        .join(&section)
        .join(&section_file)
        .with_extension(&section_ext);
    return filename_to_make(&path.to_string_lossy());
}

/// Extract all child nodes from an elment in a list.
/// If an element has multiple fields, they are concatenated
/// in a semantically useful order.
pub fn extract_content<'a>(root: Element) -> Option<Vec<Element>> {
    match root {
        Element::Document { content, .. } => Some(content),
        Element::Heading { mut caption, mut content, .. } => {
            caption.append(&mut content);
            Some(caption)
        },
        Element::Formatted { content, .. } => Some(content),
        Element::Paragraph { content, .. } => Some(content),
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
        Element::ListItem { content, .. } => Some(content),
        Element::List { content, .. } => Some(content),
        Element::Table { mut caption, mut rows, .. } => {
            caption.append(&mut rows);
            Some(caption)
        }
        Element::TableRow { cells, .. } => Some(cells),
        Element::TableCell { content, .. } => Some(content),
        Element::HtmlTag { content, .. } => Some(content),
        Element::Text { .. } => None,
        Element::Comment { .. } => None,
        Element::Error { .. } => None,
    }
}
