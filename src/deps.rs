use std::io;
use settings::Settings;
use mediawiki_parser::ast::*;
use util::*;
use std::path;


pub fn export_article_deps<'a>(root: &'a Element,
                               path: &mut Vec<&'a Element>,
                               settings: &Settings,
                               out: &mut io::Write) -> io::Result<()> {

    collect_article_deps(root, path, settings, out)?;
    collect_included_section(root, path, settings, out)
}

/// Collects the sections included in a document. At this stage, sections
/// are already inlined in the AST, but a marker comment is produced
/// for dependency extraction.
pub fn collect_included_section<'a>(root: &'a Element,
                                    path: &mut Vec<&'a Element>,
                                    settings: &Settings,
                                    out: &mut io::Write) -> io::Result<()> {

    let prefix = &settings.deps_settings.section_inclusion_prefix;
    match root {
        &Element::Comment { ref text, .. } => {
            if text.starts_with(prefix) {
                let text = trim_prefix(text, prefix);
                let mut fragments = text.split("|");

                // comment must contain article and section name
                let article = fragments.next();
                let section = fragments.next();

                if article.is_some() && section.is_some() {

                    let mut section_file = settings.deps_settings.section_rev.clone();
                    section_file.push('.');
                    section_file.push_str(&settings.deps_settings.section_ext);

                    let ipath = path::Path::new(&settings.deps_settings.section_path)
                        .join(article.unwrap())
                        .join(section.unwrap())
                        .join(&section_file);
                    let ipath = String::from(ipath.to_string_lossy());
                    writeln!(out, "{}", &filename_to_make(&ipath))?;
                }
            }
        },
        _ => traverse_with(&collect_included_section, root, path, settings, out)?,
    };
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
