use std::io;
use settings::Settings;
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


/// Function signature for export traversal.
pub type TravFunc<'a> = fn(&'a Element,
                           &mut Vec<&'a Element>,
                           &Settings,
                           &mut io::Write) -> io::Result<()>;

/// Traverse a list of subtrees with a given function.
pub fn traverse_vec<'a>(func: TravFunc<'a>,
                    content: &'a Vec<Element>,
                    path: &mut Vec<&'a Element>,
                    settings: &Settings,
                    out: &mut io::Write) -> io::Result<()> {

    for elem in &content[..] {
        func(elem, path, settings, out)?;
    }
    Ok(())
}

/// Traverse a syntax tree depth-first with a given function.
pub fn traverse_with<'a>(func: TravFunc<'a>,
                         root: &'a Element,
                         path: &mut Vec<&'a Element>,
                         settings: &Settings,
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
