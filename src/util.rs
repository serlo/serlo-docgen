use std::io;
use mediawiki_parser::ast::*;
use settings::Target;

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

/// Implements a version over a tree of `Element`.
/// All fields of the traversion struct can be mutated,
/// external settings cannot.
pub trait Traversion<'a, S: Copy> {
   /// push to the traversion path.
    fn path_push(&mut self, &'a Element);
    /// pop from the traversion path.
    fn path_pop(&mut self) -> Option<&'a Element>;
    /// get the traversion path.
    fn get_path(&self) -> &Vec<&'a Element>;
    /// template method for handling single nodes.
    /// if the result is `false`, handling is complete
    /// children of this node are not considered,
    /// otherwise `work()` is recursively called for all children.
    fn work(&mut self,
            root: &'a Element,
            settings: S,
            out: &mut io::Write) -> io::Result<bool>;

    /// run this traversion for a vector of elements.
    fn run_vec(&mut self,
               content: &'a Vec<Element>,
               settings: S,
               out: &mut io::Write) -> io::Result<()> {
        for elem in &content[..] {
            self.run(elem, settings, out)?;
        }
        Ok(())
    }
    /// run this traversion for an element.
    fn run(&mut self,
           root: &'a Element,
           settings: S,
           out: &mut io::Write) -> io::Result<()> {

        self.path_push(root);

        // break if work function breaks recursion.
        if !self.work(root, settings, out)? {
            return Ok(());
        }
        match root {
            &Element::Document { ref content, .. } => {
                self.run_vec(content, settings, out)?;
            }
            &Element::Heading {
                ref caption,
                ref content,
                ..
            } => {
                self.run_vec(caption, settings, out)?;
                self.run_vec(content, settings, out)?;
            }
            &Element::Text { .. } => (),
            &Element::Formatted { ref content, .. } => {
                self.run_vec(content, settings, out)?;
            }
            &Element::Paragraph { ref content, .. } => {
                self.run_vec(content, settings, out)?;
            }
            &Element::Template { ref content, ref name, .. } => {
                self.run_vec(content, settings, out)?;
                self.run_vec(name, settings, out)?;
            }
            &Element::TemplateArgument { ref value, .. } => {
                self.run_vec(value, settings, out)?;
            }
            &Element::InternalReference {
                ref target,
                ref options,
                ref caption,
                ..
            } => {
                self.run_vec(target, settings, out)?;
                for option in options {
                    self.run_vec(option, settings, out)?;
                }
                self.run_vec(caption, settings, out)?;
            }
            &Element::ExternalReference { ref caption, .. } => {
                self.run_vec(caption, settings, out)?;
            }
            &Element::ListItem { ref content, .. } => {
                self.run_vec(content, settings, out)?;
            }
            &Element::List { ref content, .. } => {
                self.run_vec(content, settings, out)?;
            }
            &Element::Table {
                ref caption,
                ref rows,
                ..
            } => {
                self.run_vec(caption, settings, out)?;
                self.run_vec(rows, settings, out)?;
            }
            &Element::TableRow { ref cells, .. } => {
                self.run_vec(cells, settings, out)?;
            }
            &Element::TableCell { ref content, .. } => {
                self.run_vec(content, settings, out)?;
            }
            &Element::Comment { .. } => (),
            &Element::HtmlTag { ref content, .. } => {
                self.run_vec(content, settings, out)?;
            },
            &Element::Error { .. } => (),
        }
        self.path_pop();
        Ok(())
    }
}
