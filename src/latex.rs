//! Implementation of the `latex` target.
//!
//! This target renders the final syntax tree to a LaTeX document body.
//! LaTeX boilerplate like preamble or document tags have to be added afterwards.

use std::io;
use std::io::Write;
use std::str;
use settings::Settings;
use mediawiki_parser::ast::*;
use mediawiki_parser::transformations::*;
use util::*;
use std::path;
use std::collections::HashMap;
use config;


/// This macro contains all the boilerplate code needed for a
/// non-leaf node.
macro_rules! node_template {
    (fn $name:ident ($root:ident, $path:ident, $settings:ident, $out:ident):
     $node_pattern:pat => $code:block) => {

        fn $name<'a>($root: &'a Element,
                     $path: &mut Vec<&'a Element>,
                     $settings: &Settings,
                     $out: &mut io::Write) -> io::Result<()> {

            match $root {
                $node_pattern => $code,
                _ => panic!(concat!(stringify!($name)," was called \
                    with an element it did not match! This should not \
                    happen!")),
            };
            Ok(())
        }
    }
}

node_template! {
    fn export_template(root, path, settings, out):

    &Element::Template { ref name, ref content, ref position } => {
        let template_name;
        if let Some(&Element::Text { ref text, .. }) = name.first() {
            template_name = text;
        } else {
            return write_error("Template names must be text-only!", settings, out);
        };

        let doctitle: String = setting!(settings.document_title);
        let envs: HashMap<String, Vec<String>> = setting!(settings.targets.latex.environments);

        // export simple environment templates
        if let Some(envs) = envs.get(template_name) {
            let title_content = find_arg(content, "title");

            writeln!(out, "\n% defined in {} at {}:{} to {}:{}", &doctitle,
                   position.start.line, position.start.col,
                   position.end.line, position.end.col)?;

            for environment in envs {
                if let Some(env_content) = find_arg(content, environment) {
                    path.push(env_content);
                    write!(out, "\\begin{{{}}}[", environment)?;
                    if let Some(title_content) = title_content {
                        traverse_with(&traverse_article, title_content, path, settings, out)?;
                    }
                    write!(out, "]\n")?;

                    traverse_with(&traverse_article, env_content, path, settings, out)?;
                    write!(out, "\\end{{{}}}\n", environment)?;
                    path.pop();
                }
            }
            return Ok(());
        }

        // any other template
        match &template_name[..] {
            "formula" => {
                let mut math_text = "ERROR: Template was not transformed properly!";
                if let Some(&Element::TemplateArgument { ref value, .. }) = content.first() {
                    if let Some(&Element::Text {ref text, .. }) = value.first() {
                        math_text = trim_enclosing(text.trim(),
                                                   "\\begin{align}",
                                                   "\\end{align}");
                        math_text = trim_enclosing(math_text,
                                                   "\\begin{align*}",
                                                   "\\end{align*}").trim();
                    };
                };
                let indent: usize = setting!(settings.targets.latex.indentation_depth);
                let width: usize = setting!(settings.targets.latex.max_line_width);
                writeln!(out, "{}", "\\begin{align*}")?;
                writeln!(out, "{}", indent_and_trim(math_text, indent, width))?;
                writeln!(out, "{}", "\\end{align*}")?;
            },
            _ => {
                let message = format!("MISSING TEMPLATE: {}\n{} at {}:{} to {}:{}",
                                      template_name, &doctitle,
                                      position.start.line, position.start.col,
                                      position.end.line, position.end.col);
                write_error(&message, settings, out)?;
            }
        };
    }
}

node_template! {
    fn export_internal_ref(root, path, settings, out):

    &Element::InternalReference { ref target, ref options, ref caption, ref position } => {
        let target_str = extract_plain_text(target);
        let file_ext = target_str.split(".").last().unwrap_or("").to_lowercase();

        let doctitle: String = setting!(settings.document_title);
        let img_exts: Vec<String> = setting!(settings.targets.deps.image_extensions);
        let image_path: String = setting!(settings.targets.deps.image_path);

        writeln!(out, "\n% defined in {} at {}:{} to {}:{}", &doctitle,
                   position.start.line, position.start.col,
                   position.end.line, position.end.col)?;

        // file is an image
        if img_exts.contains(&file_ext) {

            let width: f32 = setting!(settings.targets.latex.image_width);
            let height: f32 = setting!(settings.targets.latex.image_height);
            let indent: usize = setting!(settings.targets.latex.indentation_depth);
            let line_width: usize = setting!(settings.targets.latex.max_line_width);

            let image_path = path::Path::new(&image_path)
                .join(target_str)
                .file_stem()
                .expect("image path is empty!")
                .to_string_lossy()
                .to_string();
            let image_path = filename_to_make(&image_path);

            // collect image options
            let mut image_options = vec![];
            for option in options {
                image_options.push(extract_plain_text(&option).trim().to_string());
            }

            writeln!(out, "\\begin{{figure}}[h]")?;

            // render caption content
            let mut cap_content = vec![];
            writeln!(&mut cap_content, "% image options: {:?}", &image_options)?;
            writeln!(&mut cap_content, "\\adjincludegraphics[max width={}\\textwidth, \
                                          max height={}\\textheight]{{{}}}",
                width, height, &image_path)?;
            write!(&mut cap_content, "\\caption{{")?;
            traverse_vec(&traverse_article, caption, path, settings, &mut cap_content)?;
            write!(&mut cap_content, "}}")?;

            writeln!(out, "{}", &indent_and_trim(&str::from_utf8(&cap_content).unwrap(),
                indent, line_width))?;
            writeln!(out, "\\end{{figure}}")?;

            return Ok(())
        }

        write_error(&format!("No export function defined for target {:?}", target_str),
                    settings, out)?;
    }
}


node_template! {
    fn export_paragraph(root, path, settings, out):

    &Element::Paragraph { ref content, .. } => {

        // render paragraph content
        let mut par_content = vec![];
        traverse_vec(&traverse_article, content, path, settings, &mut par_content)?;
        let par_string = str::from_utf8(&par_content).unwrap().trim_right().to_string();

        let indent: usize = setting!(settings.targets.latex.indentation_depth);
        let line_width: usize = setting!(settings.targets.latex.max_line_width);

        // trim and indent output string
        let trimmed = indent_and_trim(&par_string, indent, line_width);
        writeln!(out, "{}\n", &trimmed)?;

    }
}

node_template! {
    fn export_heading(root, path, settings, out):

    &Element::Heading {ref depth, ref caption, ref content, .. } => {

        write!(out, "\\")?;

        for _ in 1..*depth {
            write!(out, "sub")?;
        }

        write!(out, "section{{")?;
        traverse_vec(&traverse_article, caption, path, settings, out)?;
        write!(out, "}}\n\n")?;

        traverse_vec(&traverse_article, content, path, settings, out)?;
    }
}

node_template! {
    fn export_list(root, path, settings, out):

    &Element::List { ref content, .. } => {

        let kind = if let &Element::ListItem { ref kind, .. } =
            content.first().unwrap_or(root) {
                kind
        } else {
            return write_error("first child of list element \
                is not a list item!", settings, out);
        };

        let envname = match kind {
            &ListItemKind::Ordered => "enumerate",
            &ListItemKind::Unordered => "itemize",
            &ListItemKind::Definition => "itemize",
            &ListItemKind::DefinitionTerm => "itemize",
        };
        writeln!(out, "\\begin{{{}}}", envname)?;

        let mut def_term_temp = String::new();

        for child in content {
            if let &Element::ListItem { ref content, ref kind, .. } = child {

                // render paragraph content
                let mut par_content = vec![];
                traverse_vec(&traverse_article, content, path, settings, &mut par_content)?;
                let par_string = str::from_utf8(&par_content).unwrap().trim_right().to_string();

                // definition term
                if let &ListItemKind::DefinitionTerm = kind {
                    def_term_temp.push_str(&par_string);
                    continue
                }

                let item_string = if let &ListItemKind::Definition = kind {
                    format!("\\item \\textbf{{{}}}: {}", def_term_temp, par_string)
                } else {
                    format!("\\item {}", par_string)
                };
                def_term_temp = String::new();


                let indent: usize = setting!(settings.targets.latex.indentation_depth);
                let line_width: usize = setting!(settings.targets.latex.max_line_width);

                // trim and indent output string
                let trimmed = indent_and_trim(&item_string, indent, line_width);

                writeln!(out, "{}", &trimmed)?;
            }
        }
        writeln!(out, "\\end{{{}}}\n", envname)?;
    }
}

node_template! {
    fn export_formatted(root, path, settings, out):

    &Element::Formatted { ref markup, ref content, .. } => {
        match markup {
            &MarkupType::NoWiki => {
                traverse_vec(&traverse_article, content, path, settings, out)?;
            },
            &MarkupType::Bold => {
                write!(out, "\\textbf{{")?;
                traverse_vec(&traverse_article, content, path, settings, out)?;
                write!(out, "}}")?;
            },
            &MarkupType::Italic => {
                write!(out, "\\textit{{")?;
                traverse_vec(&traverse_article, content, path, settings, out)?;
                write!(out, "}}")?;

            },
            &MarkupType::Math => {
                write!(out, "${}$", match content.first() {
                    Some(&Element::Text {ref text, .. }) => text,
                    _ => "parse error!",
                })?;
            },
            _ => {
                write_error(&format!("MarkupType not implemented: {:?}", &markup),
                            settings, out)?;
            }
        }
    }
}

node_template! {
    fn export_comment(root, _path, _settings, out):

    &Element::Comment { ref text, .. } => {
        writeln!(out, "% {}", text)?;
    }
}


node_template! {
    fn export_text(root, _path, _settings, out):

    &Element::Text { ref text, .. } => {
        write!(out, "{}", &escape_latex(text))?;
    }
}


fn write_error(message: &str,
               settings: &Settings,
               out: &mut io::Write) -> io::Result<()> {

    let indent: usize = setting!(settings.targets.latex.indentation_depth);
    let line_width: usize = setting!(settings.targets.latex.max_line_width);


    let message = escape_latex(message);
    writeln!(out, "\\begin{{error}}")?;
    writeln!(out, "{}", indent_and_trim(&message, indent, line_width))?;
    writeln!(out, "\\end{{error}}")
}

pub fn export_article<'a>(root: &'a Element,
                          _path: &mut Vec<&'a Element>,
                          settings: &Settings,
                          out: &mut io::Write) -> io::Result<()> {

    // apply latex-specific transformations
    let mut latex_tree = root.clone();
    latex_tree = normalize_formula(latex_tree, settings)
        .expect("Could not appy LaTeX-Secific transformations!");
    traverse_article(&latex_tree, &mut vec![], settings, out)
}

/// Recursively traverse the article tree. Node-Specific exports start here.
pub fn traverse_article<'a>(root: &'a Element,
                            path: &mut Vec<&'a Element>,
                            settings: &Settings,
                            out: &mut io::Write) -> io::Result<()> {
    path.push(root);
    match root {
        // Node elements
        &Element::Heading { .. } => export_heading(root, path, settings, out)?,
        &Element::Formatted { .. } => export_formatted(root, path, settings, out)?,
        &Element::Paragraph { .. } => export_paragraph(root, path, settings, out)?,
        &Element::Template { .. } => export_template(root, path, settings, out)?,
        &Element::InternalReference { .. } => export_internal_ref(root, path, settings, out)?,
        &Element::Document { .. } => traverse_with(&traverse_article, root, path, settings, out)?,
        &Element::List { .. } => export_list(root, path, settings, out)?,

        // Leaf Elements
        &Element::Text { .. } => export_text(root, path, settings, out)?,
        &Element::Comment { .. } => export_comment(root, path, settings, out)?,
        _ => {
            write_error(&format!("export for element `{}` not implemented!",
                root.get_variant_name()), settings, out)?;
        },
    };
    path.pop();
    Ok(())
}

/// Transform a formula template argument to text-only.
pub fn normalize_formula(mut root: Element, settings: &Settings) -> TResult {
    if let &mut Element::Template { ref name, ref mut content, ref position, .. } = &mut root {
        if let Some(&Element::Text {ref text, .. }) = name.first() {
            if text == "formula" {
                let arg_error = Element::Error {
                    position: position.clone(),
                    message: "Forumla templates must have exactly one anonymous argument, \
                                which is LaTeX source code entirely enclosed in <math></math>!".to_string(),
                };

                if content.len() != 1 {
                    return Ok(arg_error);
                }
                if let Some(&mut Element::TemplateArgument {ref mut value, .. }) = content.first_mut() {
                    if value.len() != 1 {
                        return Ok(arg_error);
                    }
                    if let Some(Element::Formatted { ref markup, ref mut content, .. }) = value.pop() {
                        if content.len() != 1 || if let &MarkupType::Math = markup {false} else {true} {
                            return Ok(arg_error);
                        }
                        value.clear();
                        value.append(content);
                    } else {
                        return Ok(arg_error);
                    }
                } else {
                    return Ok(arg_error);
                }
            }
        }
    };
    recurse_inplace(&normalize_formula, root, settings)
}
