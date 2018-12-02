//! Defines the target trait.

use preamble::*;

/// Marks an exportable target type.
pub trait Target<A, S> {
    fn target_type(&self) -> TargetType;
    /// export the the ast to `out`.
    fn export(&self, root: &Element, settings: S, args: A, out: &mut io::Write) -> io::Result<()>;
}
