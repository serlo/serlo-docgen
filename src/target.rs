//! Defines the target trait.

use preamble::*;

/// Marks an exportable target type.
pub trait Target {
    /// export the the ast to `out`.
    fn export(
        &self,
        root: &Element,
        settings: &Settings,
        args: &[String],
        out: &mut io::Write,
    ) -> io::Result<()>;

    /// Get the target-specific version of a file extension.
    ///
    /// The result of this function may contain "%", which will be
    /// replaced by the original file extension.
    fn extension_for(&self, &str) -> &str;
}
