//! Defines the target trait.

use preamble::*;
use std::collections::HashMap;

/// Marks an exportable target type.
pub trait Target {
    /// export the the ast to `out`.
    fn export(
        &self,
        root: &Element,
        settings: &Settings,
        args: &Vec<String>,
        out: &mut io::Write
    ) -> io::Result<()>;
    /// does this target operate on the input tree directly or with
    /// mfnf transformations applied?
    fn do_include_sections(&self) -> bool { false }
    /// are make dependencies generated for this target?
    fn get_target_extension(&self) -> &str;
    /// mapping of external file extensions to target extensions.
    /// this is useful if external dependencies should be processed by
    /// make for this target.
    fn get_extension_mapping(&self) -> &HashMap<String, String>;
    /// writes target-specific configuration / metadata to a JSON string.
    /// this can be be supplied to a template engine to
    /// do further modification of the target output.
    fn export_config_json(
        &self,
        out: &mut io::Write
    ) -> io::Result<()>;
}


