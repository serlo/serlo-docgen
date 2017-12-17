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

pub fn export_article<'a>(root: &'a Element,
                          path: &mut Vec<&'a Element>,
                          settings: &Settings,
                          out: &mut io::Write) -> io::Result<()> {

    path.push(root);
    match root {
        // Node elements
        &Element::Heading { .. } => export_heading(root, path, settings, out)?,

        // Leaf Elemenfs
        &Element::Text { .. } => export_text(root, out)?,

        // TODO: Remove when implementation for all elements exists
        _ => traverse_with(export_article, root, path, settings, out)?,
    };
    path.pop();
    Ok(())
}


node_template! {
    fn export_heading(root, path, settings, out):

    &Element::Heading {ref depth, ref caption, ref content, .. } => {

        write!(out, "\n\n\\")?;

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
