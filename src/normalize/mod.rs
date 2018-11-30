//! Implements the `normalize` target.
//!
//! This target is more a transformation than an export target. The output
//! is the article with normalizing transformations applied.

mod transformations;

use mediawiki_parser::transformations::TResult;
use preamble::*;
use std::process;

/// Applies some normalization transformations to an article
/// and outputs its AST as JSON.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct NormalizeTarget {}

/// Applies all transformations which should happen before section transclusion.
pub fn normalize(mut root: Element, settings: &Settings) -> TResult {
    root = transformations::normalize_template_names(root, settings)?;
    root = mwparser_utils::transformations::convert_template_list(root)?;
    if let Some(ref checker) = settings.runtime.tex_checker {
        root = mwparser_utils::transformations::normalize_math_formulas(root, checker)?;
    }
    root = transformations::remove_whitespace_trailers(root, settings)?;
    root = transformations::remove_empty_arguments(root, settings)?;
    root = transformations::resolve_interwiki_links(root, settings)?;
    root = transformations::unpack_template_arguments(root, settings)?;
    Ok(root)
}

impl Target for NormalizeTarget {
    fn extension_for(&self, _ext: &str) -> &str {
        "%"
    }

    fn export<'a>(
        &self,
        root: &'a Element,
        settings: &Settings,
        _args: &[String],
        out: &mut io::Write,
    ) -> io::Result<()> {
        let root = root.clone();

        if !settings.runtime.tex_checker.is_none() {
            eprintln!("Warning: no texvccheck path, won't perform checks!");
        }

        match normalize(root, settings) {
            Ok(root) => serde_json::to_writer(out, &root).expect("could not serialize result!"),
            Err(err) => {
                eprintln!("{}", &err);
                serde_json::to_writer(out, &err).expect("could not serialize error!");
                process::exit(1);
            }
        };
        Ok(())
    }
}
