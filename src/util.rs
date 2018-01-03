use std::io;
use mediawiki_parser::ast::*;

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

/// Trim one pair of prefix and suffix from a string.
pub fn trim_enclosing<'a>(input: &'a str, prefix: &str, suffix: &str) -> &'a str {
    if input.starts_with(prefix) && input.ends_with(suffix) {
        return &input[prefix.len()..input.len()-suffix.len()];
    }
    input
}

/// Remove a prefix if found.
pub fn trim_prefix<'a>(input: &'a str, prefix: &str) -> &'a str {
    if input.starts_with(prefix) {
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

        if line.trim().len() > max_line_width {

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
            _ => (),
        };
    }
    result
}

/// Convert a filename to a make-friedly format.
pub fn filename_to_make(input: &str) -> String {
    input.replace(" ", "_").replace(":", "@COLON@")
}

/// Function signature for export traversal.
pub type TravFunc<'a, S> = Fn(&'a Element,
                           &mut Vec<&'a Element>,
                           S,
                           &mut io::Write) -> io::Result<()>;

/// Traverse a list of subtrees with a given function.
pub fn traverse_vec<'a, S: Copy>(func: &TravFunc<'a, S>,
                    content: &'a Vec<Element>,
                    path: &mut Vec<&'a Element>,
                    settings: S,
                    out: &mut io::Write) -> io::Result<()> {

    for elem in &content[..] {
        func(elem, path, settings, out)?;
    }
    Ok(())
}

/// List element variant names.
pub fn get_path_names<'a>(path: &Vec<&'a Element>) -> Vec<&'a str> {
    let mut names = vec![];
    for elem in path {
        names.push(elem.get_variant_name());
    }
    names
}

/// Traverse a syntax tree depth-first with a given function.
pub fn traverse_with<'a, S: Copy>(func: &TravFunc<'a, S>,
                         root: &'a Element,
                         path: &mut Vec<&'a Element>,
                         settings: S,
                         out: &mut io::Write) -> io::Result<()> {

    let vec_func = traverse_vec;
    match root {
        &Element::Document { ref content, .. } => {
            vec_func(func, content, path, settings, out)?;
        }
        &Element::Heading {
            ref caption,
            ref content,
            ..
        } => {
            vec_func(func, caption, path, settings, out)?;
            vec_func(func, content, path, settings, out)?;
        }
        &Element::Text { .. } => (),
        &Element::Formatted { ref content, .. } => {
            vec_func(func, content, path, settings, out)?;
        }
        &Element::Paragraph { ref content, .. } => {
            vec_func(func, content, path, settings, out)?;
        }
        &Element::Template { ref content, ref name, .. } => {
            vec_func(func, content, path, settings, out)?;
            vec_func(func, name, path, settings, out)?;
        }
        &Element::TemplateArgument { ref value, .. } => {
            vec_func(func, value, path, settings, out)?;
        }
        &Element::InternalReference {
            ref target,
            ref options,
            ref caption,
            ..
        } => {
            vec_func(func, target, path, settings, out)?;
            for option in options {
                vec_func(func, option, path, settings, out)?;
            }
            vec_func(func, caption, path, settings, out)?;
        }
        &Element::ExternalReference { ref caption, .. } => {
            vec_func(func, caption, path, settings, out)?;
        }
        &Element::ListItem { ref content, .. } => {
            vec_func(func, content, path, settings, out)?;
        }
        &Element::List { ref content, .. } => {
            vec_func(func, content, path, settings, out)?;
        }
        &Element::Table {
            ref caption,
            ref rows,
            ..
        } => {
            vec_func(func, caption, path, settings, out)?;
            vec_func(func, rows, path, settings, out)?;
        }
        &Element::TableRow { ref cells, .. } => {
            vec_func(func, cells, path, settings, out)?;
        }
        &Element::TableCell { ref content, .. } => {
            vec_func(func, content, path, settings, out)?;
        }
        &Element::Comment { .. } => (),
        &Element::HtmlTag { ref content, .. } => {
            vec_func(func, content, path, settings, out)?;
        },
        &Element::Error { .. } => (),
    }
    Ok(())
}
