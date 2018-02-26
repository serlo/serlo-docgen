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

// --- Environments ---

alias!(GENERIC_ENV, "\
\\begin{{{}}}{}
{}
\\end{{{}}}
");

// --- Internal References ---

alias!(FIGURE_CONTENT, "\
% image options: {:?}
\\adjincludegraphics[max width={}\\textwidth, max height={}\\textheight]{{{}}}
\\caption{{{}}}\
");

alias!(INTERNAL_HREF, "\\href{{{}}}{{\\emph{{{}}}}}");

// --- HTML Elements ---

alias!(HTML_ITALIC, "\\textit{{{}}}");
alias!(HTML_REF, "\\footnote{{{}}}");

// --- Lists ---

alias!(ITEM, "\\item {}");
alias!(ITEM_DEFINITION, "\\item \\textbf{{{}}}: {}");
alias!(LIST, "\
\\begin{{{}}}
{}
\\end{{{}}}\
");

// --- Headings ---

alias!(SECTION, "\\{}section{{{}}}
");

// --- Formatting ---

alias!(BOLD, "\\textbf{{{}}}");
alias!(ITALIC, "\\textit{{{}}}");
alias!(MATH, "${}$");
alias!(STRIKE_THROUGH, "\\sout{{{}}}");
alias!(UNDERLINE, "\\ul{{{}}}");

// --- Templates ---

alias!(MATH_ENV, "align*");

// --- Galleries ---

alias!(GALLERY, "\
\\begin{{tabularx}}{{\\linewidth}}{{{}}}
{}
\\end{{tabularx}}
");

alias!(GALLERY_CONTENT, "\
% image options: {:?}
\\stepcounter{{imagelabel}}
\\addxcontentsline{{lof}}{{section}}[]{{License Info not yet supported.}}
\\begin{{minipage}}[t]{{}}
    \\begin{{figure}}[H]
        \\begin{{minipage}}[t][{}\\textheight][c][\\linewidth]
            \\centering
            \\adjincludegraphics[max width=1.\\linewidth,
                max height={}\\textheight]{{{}}}
        \\end{{minipage}}
        \\caption*{{{} (\\arabic{{imagelabel}})}}
    \\end{{figure}}
\\end{{minipage}}
");
