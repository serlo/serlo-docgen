//! This file keeps blobs of LaTeX source code in one place.


/// Creates a macro aliasing the input literal
/// to allow for compile-time subsitution.
macro_rules! alias {
    ($name:ident, $lit:expr) => {
        #[macro_export]
        macro_rules! $name {
            () => {
                $lit
            }
        }
    }
}

alias!(BLOB_FIGURE_ENV, "\
\\begin{{figure}}[h]
    % image options: {:?}
    \\adjincludegraphics[max width={}\\textwidth, max height={}\\textheight]{{{}}}
    \\caption{{{}}}
\\end{{figure}}
");
