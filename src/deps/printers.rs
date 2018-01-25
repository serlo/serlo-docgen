//! Helpers which look for certain things in the input ast and print
//! them to a given output in `make` dependency format.

use std::collections::HashMap;
use std::path::PathBuf;
use preamble::*;

/// Prints paths of the sections included in a document.
#[derive(Default)]
pub struct InclusionPrinter<'b> {
    pub path: Vec<&'b Element>,
}

impl<'a, 'b: 'a> Traversion<'a, &'b Settings> for InclusionPrinter<'a> {
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

/// Print paths of file dependencies of an article.
pub struct FilesPrinter<'a, 'b> {
    pub path: Vec<&'b Element>,
    /// map of original to target file extension of a dependency.
    pub extension_map: &'a HashMap<String, String>,
}

impl<'a, 'b: 'a> Traversion<'a, &'b Settings> for FilesPrinter<'b, 'a> {
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

impl<'a, 'b> FilesPrinter<'a, 'b> {
    pub fn new(extension_map: &'a HashMap<String, String>) -> FilesPrinter {
        FilesPrinter {
            path: vec![],
            extension_map,
        }
    }
}
