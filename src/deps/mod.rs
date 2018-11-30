//! Implementation of the `deps` target.
//!
//! The `deps` target is used to export a list of article dependencies.
//! It is applied to a syntax tree with only part of the export transformations applied.
//! Transformations such as section inclusion or heading depth normalization are excluded,
//! while others (e.g. tepmlate name translation, image prefix removal) are applied before
//! this target is executed.

use preamble::*;
use std::process;

mod printers;

use self::printers::*;
use transformations;

#[derive(Debug, Copy, Clone)]
enum PrinterKind {
    Sections,
    Media,
}

fn run_deps_printer(
    printer_kind: PrinterKind,
    root: &Element,
    settings: &Settings,
    args: &[String],
    out: &mut io::Write,
) -> io::Result<()> {
    // check of supplied targets, throw an error if target is not found.
    let mut target_list = args.to_vec();

    for (target_name, target) in &settings.general.targets {
        let target = target.get_target();
        if !args.contains(&target_name) {
            continue;
        }
        target_list = target_list
            .iter()
            .filter(|s| s != &target_name)
            .map(|s| s.clone())
            .collect();

        let docrev = &settings.runtime.document_revision;

        writeln!(out, "# dependencies for {}", &target_name)?;
        match printer_kind {
            PrinterKind::Sections => {
                write!(
                    out,
                    "{0}{1}.sections: ",
                    &settings.general.base_path.to_string_lossy(),
                    &docrev
                )?;
                let mut printer = InclusionPrinter::default();
                printer.run(&root, settings, out)?;
            }
            PrinterKind::Media => {
                write!(
                    out,
                    "{0}{1}.media: ",
                    &settings.general.base_path.to_string_lossy(),
                    &docrev
                )?;
                let mut printer = FilesPrinter::new(target);
                printer.run(&root, settings, out)?;
            }
        };
        writeln!(out)?;
    }

    if !target_list.is_empty() {
        eprintln!(
            "The following targets are not defined: {}",
            &target_list.join(", ")
        );
        process::exit(2);
    }
    Ok(())
}

/// Writes a list of included sections in `make` format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SectionDepsTarget {}

impl Target for SectionDepsTarget {
    fn extension_for(&self, _ext: &str) -> &str {
        "%"
    }

    /// Extract dependencies from a raw source AST. Sections are
    /// not included at this point.
    fn export<'a>(
        &self,
        root: &'a Element,
        settings: &Settings,
        args: &[String],
        out: &mut io::Write,
    ) -> io::Result<()> {
        // apply exclusions
        let root = transformations::remove_exclusions(root.clone(), settings)
            .expect("error applying exclusions!");
        run_deps_printer(PrinterKind::Sections, &root, settings, args, out)
    }
}

/// Writes a list of included media files in `make` format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MediaDepsTarget {}

impl Target for MediaDepsTarget {
    fn extension_for(&self, _ext: &str) -> &str {
        "%"
    }

    /// Extract dependencies from a raw source AST. Sections are
    /// not included at this point.
    fn export<'a>(
        &self,
        root: &'a Element,
        settings: &Settings,
        args: &[String],
        out: &mut io::Write,
    ) -> io::Result<()> {
        run_deps_printer(PrinterKind::Media, root, settings, args, out)
    }
}
