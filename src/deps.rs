use std::io;
use settings::Settings;
use mediawiki_parser::ast::*;
use util::*;
use std::path;
use sections::collect_section_names;


pub fn export_article_deps<'a>(root: &'a Element,
                               path: &mut Vec<&'a Element>,
                               settings: &Settings,
                               out: &mut io::Write) -> io::Result<()> {

    collect_article_deps(root, path, settings, out)?;

    let sections = collect_section_names(root, settings);
    for section in sections {
        let spath = path::Path::new(&settings.deps_settings.section_path)
            .join(&section);
        let spath = String::from(spath.to_string_lossy());
        writeln!(out, "{}", &filename_to_make(&spath))?;
    }
    Ok(())
}

fn collect_article_deps<'a>(root: &'a Element,
                                path: &mut Vec<&'a Element>,
                                settings: &Settings,
                                out: &mut io::Write) -> io::Result<()> {

    match root {
        &Element::InternalReference { ref target, .. } => {
            let target = extract_plain_text(target);
            let ext = target.split(".").last().unwrap_or("").to_lowercase();

            if settings.deps_settings.image_extensions.contains(&ext) {
                let ipath = path::Path::new(&settings.deps_settings.image_path)
                    .join(&target);
                let ipath = String::from(ipath.to_string_lossy());
                writeln!(out, "{}", &filename_to_make(&ipath))?;
            }
        },
        _ => traverse_with(&collect_article_deps, root, path, settings, out)?,
    };

    Ok(())
}
