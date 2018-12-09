//! Implements the `normalize` target.
//!
//! This target is more a transformation than an export target. The output
//! is the article with normalizing transformations applied.

mod transformations;

use crate::preamble::*;
use mediawiki_parser::transformations::TResult;
use std::path::PathBuf;
use std::process;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct NormalizeArgs {
    /// Path to the texvccheck binary (formula checking).
    #[structopt(parse(from_os_str), short = "p", long = "texvccheck-path")]
    texvccheck_path: Option<PathBuf>,
}

/// Applies some normalization transformations to an article
/// and outputs its AST as JSON.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct NormalizeTarget {}

/// Applies all transformations which should happen before section transclusion.
pub fn normalize(mut root: Element, settings: &Settings, checker: &dyn TexChecker) -> TResult {
    root = transformations::normalize_template_names(root, ())?;
    root = mwparser_utils::transformations::convert_template_list(root)?;
    root = mwparser_utils::transformations::normalize_math_formulas(root, checker)?;
    root = transformations::remove_whitespace_trailers(root, ())?;
    root = transformations::remove_empty_arguments(root, ())?;
    root = transformations::resolve_interwiki_links(root, settings)?;
    root = transformations::unpack_template_arguments(root, ())?;
    Ok(root)
}

impl<'a, 's> Target<&'a NormalizeArgs, &'s Settings> for NormalizeTarget {
    fn target_type(&self) -> TargetType {
        TargetType::Normalize
    }
    fn export(
        &self,
        root: &Element,
        settings: &'s Settings,
        args: &'a NormalizeArgs,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let root = root.clone();

        let checker = match args.texvccheck_path {
            Some(ref path) => CachedTexChecker::new(&path, 10_000),
            _ => panic!("error: no texvccheck path given, cannot normalize math!"),
        };

        match normalize(root, &settings, &checker) {
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
