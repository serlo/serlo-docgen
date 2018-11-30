//! Implements the `sections` target which writes out parts of the syntax tree.
//!
//! This target operates on the same syntax tree as the `deps` target. It extracts
//! parts of the document tree marked by `<section />` tags and writes them to a
//! directory specified through the transformation settings in the YAML format.

use preamble::*;

use serde_json;
use std::fs::DirBuilder;
use std::fs::File;
use std::io;
use std::io::Write;

mod filter;
mod finder;

/// Write marked document section to the filesystem.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct SectionsTarget {}

impl Target for SectionsTarget {
    fn include_sections(&self) -> bool {
        false
    }
    fn target_extension(&self) -> &str {
        "json"
    }
    fn extension_for(&self, _ext: &str) -> &str {
        "%"
    }
    fn export<'a>(
        &self,
        root: &'a Element,
        settings: &Settings,
        _: &[String],
        _: &mut io::Write,
    ) -> io::Result<()> {
        let sections = finder::SectionNameCollector::collect_from(root);

        for section in sections {
            if section.is_empty() {
                continue;
            }

            let inter = filter::SectionFilter::extract(&section, root);

            let mut filename = settings.runtime.document_revision.clone();
            let file_ext = &settings.general.section_ext;

            filename.push('.');
            filename.push_str(file_ext);

            let mut path = settings
                .general
                .section_path
                .join(&filename_to_make(&section));

            DirBuilder::new().recursive(true).create(&path)?;

            path = path.join(&filename);

            let mut file = File::create(&path)?;
            file.write_all(
                serde_json::to_string(&inter)
                    .expect("could not serialize section!")
                    .as_bytes(),
            )?;
        }
        Ok(())
    }
}
