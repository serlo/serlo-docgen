//! Implementation of the `deps` target.
//!
//! The `deps` target is used to export a list of article dependencies.
//! It is applied to a syntax tree with only part of the export transformations applied.
//! Transformations such as section inclusion or heading depth normalization are excluded,
//! while others (e.g. tepmlate name translation, image prefix removal) are applied before
//! this target is executed.

use std::io;
use settings::Settings;
use mediawiki_parser::ast::*;
use util::*;
use std::path;


/// Extract dependencies from a RAW source AST. Sections are
/// not included at this point.
pub fn export_article_deps<'a>(root: &'a Element,
                               path: &mut Vec<&'a Element>,
                               settings: &Settings,
                               out: &mut io::Write) -> io::Result<()> {

    collect_article_deps(root, path, settings, out)?;
    collect_included_section(root, path, settings, out)
}

/// Collects the sections included in a document.
pub fn collect_included_section<'a>(root: &'a Element,
                                    path: &mut Vec<&'a Element>,
                                    settings: &Settings,
                                    out: &mut io::Write) -> io::Result<()> {

    if let &Element::Template { ref name, ref content, .. } = root {
        let prefix = &settings.deps_settings.section_inclusion_prefix;
        let template_name = extract_plain_text(&name);

        // section transclusion
        if template_name.to_lowercase().starts_with(prefix) {
            let article = trim_prefix(&template_name, prefix);

            let section_name = extract_plain_text(content);
            let mut section_file = settings.deps_settings.section_rev.clone();
            section_file.push('.');
            section_file.push_str(&settings.deps_settings.section_ext);

            let path = path::Path::new(&settings.deps_settings.section_path)
                .join(&filename_to_make(&article))
                .join(&filename_to_make(&section_name))
                .join(&filename_to_make(&section_file));
            writeln!(out, "{}", &filename_to_make(&path.to_string_lossy()))?;
        }
    };
    traverse_with(&collect_included_section, root, path, settings, out)
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
