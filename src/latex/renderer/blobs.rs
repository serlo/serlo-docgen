//! This file keeps blobs of LaTeX source code in one place.

/// Creates a macro aliasing the input literal
/// to allow for compile-time subsitution.
macro_rules! alias {
    ($name:ident, $lit:expr) => {
        #[macro_export]
        macro_rules! $name {
            () => {
                $lit
            };
        }
    };
}

// --- Environments ---

alias!(
    GENERIC_ENV,
    "\\begin{{{}}}{}
{}
\\end{{{}}}%"
);

// --- Internal References ---

alias!(
    LICENSE_TEXT,
    "Abb. \\arabic{{imagelabel}}: \
     \\protect\\href{{{}}}{{\\textbf{{{}}}}} by {} \\textit{{({})}}"
);

alias!(
    FIGURE_CONTENT,
    "\
% image options: {:?}
\\stepcounter{{imagelabel}}
\\centering
\\addxcontentsline{{lof}}{{section}}[]{{{}}}
\\adjincludegraphics[max width={}\\textwidth, max height={}\\textheight]{{{}}}\
"
);

alias!(FIGURE_CAPTION, "\\caption{{{} (\\arabic{{imagelabel}})}}");
alias!(
    FIGURE_INLINE,
    "
% image options: {:?}
\\stepcounter{{imagelabel}}
\\addxcontentsline{{lof}}{{section}}[]{{{}}}
\\adjincludegraphics[height=\\lineheight]{{{}}}\
"
);

alias!(INTERNAL_HREF, "\\href{{{}}}{{\\emph{{{}}}}}");

// --- HTML Elements ---

alias!(HTML_ITALIC, "\\textit{{{}}}");
alias!(HTML_REF, "\\footnote{{{}}}");

// --- Lists ---

alias!(ITEM, "\\item {}");
alias!(ITEM_DEFINITION, "\\item[{}:] {}");
alias!(
    LIST,
    "\\begin{{{}}}
{}
\\end{{{}}}"
);

// --- Headings ---

alias!(SECTION, "\\{}section{{{}}}");

// --- Formatting ---

alias!(BOLD, "\\textbf{{{}}}");
alias!(ITALIC, "\\textit{{{}}}");
alias!(MATH, "${}$");
alias!(STRIKE_THROUGH, "\\sout{{{}}}");
alias!(UNDERLINE, "\\ul{{{}}}");
alias!(QUOTE_ENV, "displayquote");

// --- Templates ---

alias!(MATH_ENV, "align");
alias!(IMPORTANT_ENV, "important");
alias!(PROOF_STEP_ENV, "proofstep");
alias!(PROOF_CASE_ENV, "proofcase");
alias!(
    INDUCTION,
    "\\textbf{{Aussageform, deren Allgemeingültigkeit für {} bewiesen werden soll:}}

{}

\\begin{{enumerate}}
\\item \\textbf{{Induktionsanfang:}} {}
\\item \\textbf{{Induktionsschritt:}}
\\begin{{enumerate}}
\\item \\textbf{{Induktionsvoraussetzung:}} {}
\\item \\textbf{{Induktionsbehauptung:}} {}
\\item \\textbf{{Beweis des Induktionsschritts:}} {}
\\end{{enumerate}}
\\end{{enumerate}}"
);
alias!(INDUCTION_SET_DEFAULT, "$n\\in\\mathcal{{N}}$");
alias!(EXERCISE_TASKLIST, "{}\n\n{}");
alias!(EXERCISE_EXPLANATION, "{}\n\n{}");

// --- Galleries ---

alias!(
    GALLERY,
    "\\begin{{figure}}
    \\centering
{}
\\end{{figure}}"
);

alias!(
    GALLERY_CONTENT,
    "\
% image options: {:?}
\\begin{{minipage}}[t]{{{}\\textwidth}}
    \\centering
    \\stepcounter{{imagelabel}}
    \\addxcontentsline{{lof}}{{section}}[]{{{}}}
    \\adjincludegraphics[max width=1.\\textwidth,
        max height={}\\textheight]{{{}}}
    \\captionof{{figure}}{{{} (\\arabic{{imagelabel}})}}
\\end{{minipage}}
"
);

// --- Table ---
alias!(
    TABLE,
    "\\renewcommand{{\\arraystretch}}{{1.5}}
\\begin{{longtabu}} to \\linewidth {{{}}}
\\caption{{{}}}\\\\ \\toprule
{}
\\bottomrule
\\end{{longtabu}}
\\renewcommand{{\\arraystretch}}{{1.0}}"
);

alias!(TABLE_WITH_HEADER, "{}\\midrule\n{}");

// --- Anchor ---
alias!(LABEL, "\\label{{{}}}");
alias!(LABEL_REF, "\\hyperref[{}]{{\\emph{{{}}}}}");

// --- Main Article ---
alias!(MAINARTICLE, "$\\rightarrow$ Hauptartikel: ");

// --- Comments ---
alias!(
    COMMENT,
    "\\begin{{comment}}
{}
\\end{{comment}}"
);
