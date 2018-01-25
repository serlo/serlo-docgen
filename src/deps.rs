//! Implementation of the `deps` target.
//!
//! The `deps` target is used to export a list of article dependencies.
//! It is applied to a syntax tree with only part of the export transformations applied.
//! Transformations such as section inclusion or heading depth normalization are excluded,
//! while others (e.g. tepmlate name translation, image prefix removal) are applied before
//! this target is executed.

use std::io;
use settings::*;
use mediawiki_parser::ast::*;
use util::*;
use std::path::PathBuf;
use std::collections::HashMap;


/// Writes a list of `make` dependencies for each target.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DepsTarget {
    extension_map_dummy: HashMap<String, String>,
}

impl Target for DepsTarget {
    fn get_name(&self) -> &str { "dependencies" }

    fn get_target_extension(&self) -> &str { "dep" }

    fn get_extension_mapping(&self) -> &HashMap<String, String> {
        &self.extension_map_dummy
    }

    /// Extract dependencies from a RAW source AST. Sections are
    /// not included at this point.
    fn export<'a>(&self, root: &'a Element,
                         path: &mut Vec<&'a Element>,
                         settings: &Settings,
                         out: &mut io::Write) -> io::Result<()> {

        let docrev = &settings.document_revision;
        for (name, target) in &settings.targets {

            let target = target.get_target();

            if !target.do_generate_dependencies() {
                continue;
            }

            let target_ext = target.get_target_extension();

            writeln!(out, "# dependencies for {}", &name)?;
            write!(out, "{}.{}:", &docrev, target_ext)?;

            let mut file_collection = FileCollectionTraversion {
                path: vec![],
                extension_map: target.get_extension_mapping(),
            };

            let mut section_collection = SectionCollectionTraversion::default();

            file_collection.run(root, settings, out)?;
            section_collection.run(root, settings, out)?;
        }
        Ok(())
    }
}

/// Collects the sections included in a document.
#[derive(Default)]
struct SectionCollectionTraversion<'b> {
    pub path: Vec<&'b Element>,
}

impl<'a, 'b: 'a> Traversion<'a, &'b Settings> for SectionCollectionTraversion<'a> {
    fn path_push(&mut self, root: &'a Element) {
        self.path.push(&root);
    }
    fn path_pop(&mut self) -> Option<&'a Element> {
        self.path.pop()
    }
    fn get_path(&self) -> &Vec<&'a Element> {
        &self.path
    }
    fn work(&mut self, root: &Element, settings: &'b Settings,
            out: &mut io::Write) -> io::Result<bool> {

        if let &Element::Template { ref name, ref content, .. } = root {
            let prefix: &str = &settings.section_inclusion_prefix;
            let template_name = extract_plain_text(&name);

            // section transclusion
            if template_name.to_lowercase().starts_with(&prefix) {
                let article = trim_prefix(&template_name, &prefix);
                let section_name = extract_plain_text(content);

                let mut section_file = settings.section_rev.clone();
                let section_ext = &settings.section_ext;
                let section_path = &settings.section_path;

                section_file.push('.');
                section_file.push_str(section_ext);

                let path = PathBuf::from(section_path)
                    .join(&article)
                    .join(&section_name)
                    .join(&section_file);
                write!(out, " \\\n\t{}", &filename_to_make(&path.to_string_lossy()))?;
            }
        };
        Ok(true)
    }
}

/// Collects file dependencies of an article.
struct FileCollectionTraversion<'a, 'b> {
    pub path: Vec<&'b Element>,
    /// map of original to target file extension of a dependency.
    pub extension_map: &'a HashMap<String, String>,
}

impl<'a, 'b: 'a> Traversion<'a, &'b Settings> for FileCollectionTraversion<'b, 'a> {
    fn path_push(&mut self, root: &'a Element) {
        self.path.push(&root);
    }
    fn path_pop(&mut self) -> Option<&'a Element> {
        self.path.pop()
    }
    fn get_path(&self) -> &Vec<&'a Element> {
        &self.path
    }
    fn work(&mut self, root: &Element, settings: &'b Settings,
            out: &mut io::Write) -> io::Result<bool> {

        if let &Element::InternalReference { ref target, .. } = root {
            let target = extract_plain_text(target);
            let target_path = PathBuf::from(target);
            let ext = target_path.extension().unwrap_or_default();
            let ext_str = ext.to_string_lossy().into();

            let extensions = &settings.image_extensions;
            let image_path = &settings.image_path;
            let target_extension = self.extension_map.get(&ext_str).unwrap_or(&ext_str);

            if extensions.contains(&ext_str) {
                let ipath = PathBuf::from(&image_path)
                    .join(&target_path)
                    .with_extension(target_extension);
                let ipath = ipath.to_string_lossy().to_string();
                write!(out, " \\\n\t{}", &filename_to_make(&ipath))?;
            }
        };
        Ok(true)
    }
}
