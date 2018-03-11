//! Implementation of the `deps` target.
//!
//! The `deps` target is used to export a list of article dependencies.
//! It is applied to a syntax tree with only part of the export transformations applied.
//! Transformations such as section inclusion or heading depth normalization are excluded,
//! while others (e.g. tepmlate name translation, image prefix removal) are applied before
//! this target is executed.

use preamble::*;
use std::collections::HashMap;

mod printers;

use self::printers::*;

/// Writes a list of `make` dependencies for each target.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DepsTarget {
    #[serde(skip_serializing_if = "is_default")]
    extension_map_dummy: HashMap<String, String>,
}

impl Target for DepsTarget {
    fn get_target_extension(&self) -> &str { "dep" }

    fn get_extension_mapping(&self) -> &HashMap<String, String> {
        &self.extension_map_dummy
    }

    /// Extract dependencies from a RAW source AST. Sections are
    /// not included at this point.
    fn export<'a>(
        &self,
        root: &'a Element,
        settings: &Settings,
        args: &Vec<String>,
        out: &mut io::Write) -> io::Result<()>
    {

        let docrev = &settings.document_revision;
        for (name, target) in &settings.targets {

            let target = target.get_target();

            if !args.contains(name) {
                continue;
            }

            let target_ext = target.get_target_extension();

            writeln!(out, "# dependencies for {}", &name)?;
            write!(out, "{}.{}:", &docrev, target_ext)?;

            let mut file_collection = FilesPrinter::new(target.get_extension_mapping());
            let mut section_collection = InclusionPrinter::default();

            file_collection.run(root, settings, out)?;
            section_collection.run(root, settings, out)?;
            writeln!(out, "")?;
        }
        Ok(())
    }

    fn export_config_json(&self, out: &mut io::Write) -> io::Result<()> {
        write!(out, "{}", "{}")
    }
}

