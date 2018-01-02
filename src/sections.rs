use std::io;
use std::str;
use settings::Settings;
use mediawiki_parser::ast::*;
use util::*;

pub fn collect_sections<'a>(root: &'a Element,
                            path: &mut Vec<&'a Element>,
                            settings: &Settings,
                            out: &mut io::Write) -> io::Result<()> {

    match root {
        &Element::HtmlTag { ref name, ref attributes, .. } => {
            if name.to_lowercase() == "section" {
                for attr in attributes {
                    eprintln!("section attr: {}={}", attr.key, attr.value);
                }
            }
        },
        _ => (),
    };
    traverse_with(collect_sections, root, path, settings, out)?;
    Ok(())
}
