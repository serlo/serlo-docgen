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

#[derive(Debug, Copy, Clone)]
enum PrinterKind {
    Sections,
    Media
}

fn run_deps_printer(
    printer_kind: PrinterKind,
    root: &Element,
    settings: &Settings,
    args: &[String],
    out: &mut io::Write
) -> io::Result<()> {

    for (target_name, target) in &settings.general.targets {
        let target = target.get_target();
        if !args.contains(&target_name) {
            continue
        }

        // apply exclusions
        let root = {
            let mut new_settings = Settings::default();
            new_settings.runtime.markers = settings.runtime.markers.clone();
            new_settings.runtime.target_name = target_name.to_string();
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

        let target_ext = target.get_target_extension();
        let docrev = &settings.runtime.document_revision;

        writeln!(out, "# dependencies for {}", &target_name)?;
        write!(out, "{}.{}: ", &docrev, target_ext)?;
        let mut printer: Box<Traversion<&Settings>> = match printer_kind {
            PrinterKind::Sections => Box::new(InclusionPrinter::default()),
            PrinterKind::Media => {
                Box::new(FilesPrinter::new(target.get_extension_mapping()))
            },
        };
        printer.run(&root, settings, out)?;
        writeln!(out, "")?;
    }
    Ok(())
}

/// Writes a list of included sections in `make` format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SectionDepsTarget {
    #[serde(skip_serializing_if = "is_default")]
    extension_map_dummy: HashMap<String, String>,
}

impl Target for SectionDepsTarget {
    fn get_target_extension(&self) -> &str { "sections" }
    fn do_include_sections(&self) -> bool { false }

    fn get_extension_mapping(&self) -> &HashMap<String, String> {
        &self.extension_map_dummy
    }

    /// Extract dependencies from a raw source AST. Sections are
    /// not included at this point.
    fn export<'a>(
        &self,
        root: &'a Element,
        settings: &Settings,
        args: &[String],
        out: &mut io::Write) -> io::Result<()>
    {
        run_deps_printer(PrinterKind::Sections, root, settings, args, out)
    }
}

/// Writes a list of included media files in `make` format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MediaDepsTarget {
    #[serde(skip_serializing_if = "is_default")]
    extension_map_dummy: HashMap<String, String>,
}

impl Target for MediaDepsTarget {
    fn get_target_extension(&self) -> &str { "media" }
    fn do_include_sections(&self) -> bool { true }

    fn get_extension_mapping(&self) -> &HashMap<String, String> {
        &self.extension_map_dummy
    }

    /// Extract dependencies from a raw source AST. Sections are
    /// not included at this point.
    fn export<'a>(
        &self,
        root: &'a Element,
        settings: &Settings,
        args: &[String],
        out: &mut io::Write) -> io::Result<()>
    {
        run_deps_printer(PrinterKind::Media, root, settings, args, out)
    }
}
