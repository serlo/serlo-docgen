use std::io;
use std::str;
use settings::Settings;
use mediawiki_parser::ast::*;
use util::*;


pub fn collect_article_deps<'a>(root: &'a Element,
                                path: &mut Vec<&'a Element>,
                                settings: &Settings,
                                out: &mut io::Write) -> io::Result<()> {

    match root {
        &Element::InternalReference { ref target, .. } => {
            eprintln!("ref {}", extract_plain_text(target));
        },
        _ => traverse_with(collect_article_deps, root, path, settings, out)?,
    };

    Ok(())
}
