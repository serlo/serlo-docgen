use std::io;
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
            return write_error("Template names must be text-only!", out);
        };

        // export simple environment templates
        if let Some(envs) = settings.latex_settings.environments.get(template_name) {
            let title_content = find_arg(content, "title");

            write!(out, "% defined in {} at {}:{} to {}:{}\n", settings.document_title,
                   position.start.line, position.start.col,
                   position.end.line, position.start.col)?;

            for environment in envs {
                if let Some(env_content) = find_arg(content, environment) {

                    write!(out, "\\begin{{{}}}[", environment)?;
                    if let Some(title_content) = title_content {
                        traverse_with(export_article, title_content, path, settings, out)?;
                    }
                    write!(out, "]\n")?;

                    traverse_with(export_article, env_content, path, settings, out)?;
                    write!(out, "\\end{{{}}}\n", environment)?;
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
                write!(out, "\\begin{{align*}}\n{}\n\\end{{align*}}\n", math_text)?;
            },
            _ => {
                write_error(&format!("MISSING TEMPLATE: {} ({}:{} to {}:{})",
                                     template_name, position.start.line, position.start.col,
                                     position.end.line, position.end.col), out)?;
            }
        };
    }
}

node_template! {
    fn export_paragraph(root, path, settings, out):

    &Element::Paragraph { ref content, .. } => {
        traverse_vec(export_article, content, path, settings, out)?;
        write!(out, "\\\\\n")?;
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

fn export_text(root: &Element, out: &mut io::Write) -> io::Result<()> {
    match root {
        &Element::Text { ref text, .. } => {
            write!(out, "{}", escape_latex(text))?;
        },
        _ => unreachable!(),
    }
    Ok(())
}


fn write_error(message: &str, out: &mut io::Write) -> io::Result<()> {
    write!(out, "\\begin{{error}}\n{}\n\\end{{error}}\n", message)
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
        &Element::Text { .. } => export_text(root, out)?,

        // TODO: Remove when implementation for all elements exists
        _ => traverse_with(export_article, root, path, settings, out)?,
    };
    path.pop();
    Ok(())
}
