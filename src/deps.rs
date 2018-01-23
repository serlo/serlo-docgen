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
use std::collections::HashMap;
use config;
use std::ffi::OsStr;

/// Extract dependencies from a RAW source AST. Sections are
/// not included at this point.
pub fn export_article_deps<'a>(root: &'a Element,
                               path: &mut Vec<&'a Element>,
                               settings: &Settings,
                               out: &mut io::Write) -> io::Result<()> {

    let docrev: String = setting!(settings.document_revision);
    // TODO use rule with multiple targets - one for each possible export
    let targets: HashMap<String, config::Value> = setting!(settings.targets);
    for (name, target) in targets {

        let gen_deps: bool = from_table!(target.generate_deps);
        if !gen_deps {
            continue;
        }

        let target_ext: String = from_table!(target.target_extension);

        writeln!(out, "# dependencies for {}", &name)?;
        write!(out, "{}.{}:", &docrev, &target_ext)?;

        let props = CollectionProps {
            extension_mapping: from_table!(target.deps_extension_mapping),
            settings: &settings,
        };

        collect_article_deps(root, path, &props, out)?;
        collect_included_section(root, path, &props, out)?;
    }
    Ok(())
}

struct CollectionProps<'a> {
    pub extension_mapping: HashMap<String, String>,
    pub settings: &'a Settings,
}

/// Collects the sections included in a document.
fn collect_included_section<'a>(root: &'a Element,
                                    path: &mut Vec<&'a Element>,
                                    props: &CollectionProps,
                                    out: &mut io::Write) -> io::Result<()> {

    let settings = props.settings;
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
    traverse_with(&collect_included_section, root, path, props, out)
}

fn collect_article_deps<'a>(root: &'a Element,
                            path: &mut Vec<&'a Element>,
                            props: &CollectionProps,
                            out: &mut io::Write) -> io::Result<()> {

    let settings = &props.settings;
    let ext_mapping = &props.extension_mapping;

    if let &Element::InternalReference { ref target, .. } = root {
        let target = extract_plain_text(target);
        let target_path = path::Path::new(&target);
        let ext = target_path.extension().unwrap_or(OsStr::new(""));
        let ext_str = ext.to_os_string().into_string().unwrap_or(String::new());

        let extensions: Vec<String> = setting!(settings.targets.deps.image_extensions);
        let image_path: String = setting!(settings.targets.deps.image_path);

        if extensions.contains(&ext_str) {
            let ipath = path::Path::new(&image_path)
                .join(&target)
                .with_extension(ext_mapping.get(&ext_str).unwrap_or(&ext_str));
            let ipath = String::from(ipath.to_string_lossy());
            write!(out, " \\\n\t{}", &filename_to_make(&ipath))?;
        }
    } else {
        traverse_with(&collect_article_deps, root, path, props, out)?;
    }
    Ok(())
}
