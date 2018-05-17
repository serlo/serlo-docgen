//! Implementation of the `deps` target.
//!
//! The `deps` target is used to export a list of article dependencies.
//! It is applied to a syntax tree with only part of the export transformations applied.
//! Transformations such as section inclusion or heading depth normalization are excluded,
//! while others (e.g. tepmlate name translation, image prefix removal) are applied before
//! this target is executed.

use preamble::*;
use std::collections::HashMap;
use serde_yaml;
use std::process;

mod printers;

use self::printers::*;
use transformations;

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
        args: &[String],
        out: &mut io::Write) -> io::Result<()>
    {
        let docrev = &settings.runtime.document_revision;
        for (name, target) in &settings.general.targets {

            // apply exclusions
            let root = {
                let mut new_settings = Settings::default();
                new_settings.runtime.markers = settings.runtime.markers.clone();
                new_settings.runtime.target_name = name.clone();
                let root = root.clone();
                let result = transformations::remove_exclusions(root, &new_settings);
                match result {
                    Err(err) => {
                        eprintln!("{}", &err);
                        println!("{}", serde_yaml::to_string(&err)
                            .expect("Could not serialize error!"));
                        process::exit(1);
                    }
                    Ok(tree) => tree,
                }
            };

            let target = target.get_target();

            if !args.contains(name) {
                continue;
            }

            let target_ext = target.get_target_extension();

            let mut file_collection = FilesPrinter::new(target.get_extension_mapping());
            let mut section_collection = InclusionPrinter::default();
            writeln!(out, "# dependencies for {}", &name)?;
            write!(out, "{}.{}: ", &docrev, target_ext)?;
            file_collection.run(&root, settings, out)?;
            section_collection.run(&root, settings, out)?;
            writeln!(out, "")?;
        }
        Ok(())
    }
}

