//! Implements the `sections` target which writes out parts of the syntax tree.
//!
//! This target operates on the same syntax tree as the `deps` target. It extracts
//! parts of the document tree marked by `<section />` tags and writes them to a
//! directory specified through the transformation settings in the YAML format.

use std::collections::HashMap;
use settings::*;
use mediawiki_parser::ast::*;

use util::filename_to_make;
use std::path;
use std::fs::File;
use std::io::Write;
use std::io;
use std::fs::DirBuilder;
use serde_yaml;

mod finder;
mod filter;

/// Write marked document section to the filesystem.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SectionsTarget {
    pub extension_mapping: HashMap<String, String>,
}

impl Target for SectionsTarget {
    fn get_name(&self) -> &str { "sections" }
    fn do_include_sections(&self) -> bool { false }
    fn do_generate_dependencies(&self) -> bool { false }
    fn get_target_extension(&self) -> &str { "yml" }
    fn get_extension_mapping(&self) -> &HashMap<String, String> {
        &self.extension_mapping
    }
    fn export<'a>(&self,
                root: &'a Element,
                settings: &Settings,
                _: &mut io::Write) -> io::Result<()> {

        let sections = finder::SectionNameCollector::collect_from(root);

        for section in sections {
            if section.is_empty() {
                continue
            }

            let inter = filter::SectionFilter::extract(&section, root);

            let mut filename = settings.document_revision.clone();
            let file_ext = &settings.section_ext;
            let section_path = &settings.section_path;
            let doctitle = &settings.document_title;

            filename.push('.');
            filename.push_str(&file_ext);

            let mut path = path::Path::new(&section_path)
                .join(&filename_to_make(&doctitle))
                .join(&filename_to_make(&section.clone()));

            DirBuilder::new()
                .recursive(true)
                .create(&path)?;

            path = path.join(&filename);

            let mut file = File::create(&path)?;
            file.write_all(&serde_yaml::to_string(&inter)
                .expect("could not serialize section!")
                .as_bytes())?;

        }
        Ok(())
    }
}
