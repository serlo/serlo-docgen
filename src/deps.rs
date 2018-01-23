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
use config;


/// Extract dependencies from a RAW source AST. Sections are
/// not included at this point.
pub fn export_article_deps<'a>(root: &'a Element,
                               path: &mut Vec<&'a Element>,
                               settings: &Settings,
                               out: &mut io::Write) -> io::Result<()> {

    let docrev: String = setting!(settings.document_revision);
    // TODO use rule with multiple targets - one for each possible export
    write!(out, "{}.pdf:", &docrev)?;
    collect_article_deps(root, path, settings, out)?;
    collect_included_section(root, path, settings, out)
}

/// Collects the sections included in a document.
pub fn collect_included_section<'a>(root: &'a Element,
                                    path: &mut Vec<&'a Element>,
                                    settings: &Settings,
                                    out: &mut io::Write) -> io::Result<()> {

    if let &Element::Template { ref name, ref content, .. } = root {
        let prefix: String = setting!(settings.targets.deps.section_inclusion_prefix);
        let template_name = extract_plain_text(&name);

        // section transclusion
        if template_name.to_lowercase().starts_with(&prefix) {
            let article = trim_prefix(&template_name, &prefix);
            let section_name = extract_plain_text(content);

            let mut section_file: String = setting!(settings.targets.deps.section_rev);
            let section_ext: String = setting!(settings.targets.deps.section_ext);
            let section_path: String = setting!(settings.targets.deps.section_path);

            section_file.push('.');
            section_file.push_str(&section_ext);

            let path = path::Path::new(&section_path)
                .join(&filename_to_make(&article))
                .join(&filename_to_make(&section_name))
                .join(&filename_to_make(&section_file));
            write!(out, " \\\n\t{}", &filename_to_make(&path.to_string_lossy()))?;
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

            let extensions: Vec<String> = setting!(settings.targets.deps.image_extensions);
            let image_path: String = setting!(settings.targets.deps.image_path);

            if extensions.contains(&ext) {
                let ipath = path::Path::new(&image_path)
                    .join(&target);
                let ipath = String::from(ipath.to_string_lossy());
                write!(out, " \\\n\t{}", &filename_to_make(&ipath))?;
            }
        },
        _ => traverse_with(&collect_article_deps, root, path, settings, out)?,
    };

    Ok(())
}
