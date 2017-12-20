use std::io;
use std::str;
use settings::Settings;
use mediawiki_parser::ast::*;
use util::*;


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

        // export simple environment templates
        if let Some(envs) = settings.latex_settings.environments.get(template_name) {
            let title_content = find_arg(content, "title");

            write!(out, "% defined in {} at {}:{} to {}:{}\n", settings.document_title,
                   position.start.line, position.start.col,
                   position.end.line, position.start.col)?;

            for environment in envs {
                if let Some(env_content) = find_arg(content, environment) {
                    path.push(env_content);
                    write!(out, "\\begin{{{}}}[", environment)?;
                    if let Some(title_content) = title_content {
                        traverse_with(export_article, title_content, path, settings, out)?;
                    }
                    write!(out, "]\n")?;

                    traverse_with(export_article, env_content, path, settings, out)?;
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
                let indent = settings.latex_settings.indentation_depth;
                let width = settings.latex_settings.max_line_width;
                writeln!(out, "{}", "\\begin{align*}")?;
                writeln!(out, "{}", indent_and_trim(math_text, indent, width))?;
                writeln!(out, "{}", "\\end{align*}")?;
            },
            _ => {
                let message = format!("MISSING TEMPLATE: {}\n{} at {}:{} to {}:{}",
                                      template_name, settings.document_title,
                                      position.start.line, position.start.col,
                                      position.end.line, position.end.col);
                write_error(&message, settings, out)?;
            }
        };
    }
}

node_template! {
    fn export_paragraph(root, path, settings, out):

    &Element::Paragraph { ref content, .. } => {

        // render paragraph content
        let mut par_content = vec![];
        traverse_vec(export_article, content, path, settings, &mut par_content)?;
        let par_string = str::from_utf8(&par_content).unwrap().trim_right().to_string();

        // trim and indent output string
        let trimmed = indent_and_trim(&par_string,
            settings.latex_settings.indentation_depth,
            settings.latex_settings.max_line_width);
        writeln!(out, "{}\n", &trimmed)?;

    }
}

node_template! {
    fn export_heading(root, path, settings, out):

    &Element::Heading {ref depth, ref caption, ref content, .. } => {

        write!(out, "\\")?;

        for _ in 2..*depth {
            write!(out, "sub")?;
        }

        write!(out, "section{{")?;
        traverse_vec(export_article, caption, path, settings, out)?;
        write!(out, "}}\n\n")?;

        traverse_vec(export_article, content, path, settings, out)?;
    }
}

node_template! {
    fn export_formatted(root, path, settings, out):

    &Element::Formatted { ref markup, ref content, .. } => {
        match markup {
            &MarkupType::NoWiki => {
                traverse_vec(export_article, content, path, settings, out)?;
            },
            &MarkupType::Bold => {
                write!(out, "\\textbf{{")?;
                traverse_vec(export_article, content, path, settings, out)?;
                write!(out, "}}")?;
            },
            &MarkupType::Italic => {
                write!(out, "\\textit{{")?;
                traverse_vec(export_article, content, path, settings, out)?;
                write!(out, "}}")?;

            },
            &MarkupType::Math => {
                write!(out, "${}$", match content.first() {
                    Some(&Element::Text {ref text, .. }) => text,
                    _ => "parse error!",
                })?;
            },
            _ => (),
        }
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

    let indent = settings.latex_settings.indentation_depth;
    let width = settings.latex_settings.max_line_width;
    writeln!(out, "\\begin{{error}}")?;
    writeln!(out, "{}", indent_and_trim(message, indent, width))?;
    writeln!(out, "\\end{{error}}")
}

pub fn export_article<'a>(root: &'a Element,
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

        // Leaf Elemenfs
        &Element::Text { .. } => export_text(root, path, settings, out)?,

        // TODO: Remove when implementation for all elements exists
        _ => traverse_with(export_article, root, path, settings, out)?,
    };
    path.pop();
    Ok(())
}
