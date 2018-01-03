use std::io;
use settings::Settings;
use mediawiki_parser::ast::*;
use util::*;
use std::path;

pub fn collect_article_deps<'a>(root: &'a Element,
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
