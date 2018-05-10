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

    path_methods!('a);

    fn work(&mut self, root: &Element, settings: &'b Settings,
            out: &mut io::Write) -> io::Result<bool> {

        if let Element::Template(ref template) = *root {
            let prefix: &str = &settings.section_inclusion_prefix;
            let template_name = extract_plain_text(&template.name);

            // section transclusion
            if template_name.to_lowercase().starts_with(&prefix) {
                let article = filename_to_make(&trim_prefix(&template_name, prefix));
                let section_name = filename_to_make(&extract_plain_text(&template.content));
                let path = get_section_path(&article, &section_name, settings);
                write!(out, " {}", &path)?;
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

    path_methods!('a);

    fn work(&mut self, root: &Element, settings: &'b Settings,
            out: &mut io::Write) -> io::Result<bool> {

        if let Element::InternalReference(ref iref) = *root {
            let target = filename_to_make(&extract_plain_text(&iref.target));
            let target_path = PathBuf::from(target);
            let ext = target_path.extension().unwrap_or_default();
            let ext_str = ext.to_string_lossy().into();

            let extensions = &settings.external_file_extensions;
            let file_path = &settings.external_file_path;
            let target_extension = self.extension_map.get(&ext_str).unwrap_or(&ext_str);

            if extensions.contains(&ext_str) {
                let path = PathBuf::from(&file_path)
                    .join(&target_path)
                    .with_extension(target_extension);
                let path = path.to_string_lossy().to_string();
                write!(out, " {}", &path)?;
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
