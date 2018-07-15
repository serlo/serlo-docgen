//! CLI front end for the mfnf export tool.
//!
//! Applies some transformations to the input tree and exports it as defined by the given target.

extern crate mediawiki_parser;
extern crate serde_yaml;
extern crate serde_json;
#[macro_use]
extern crate structopt;
extern crate mfnf_export;
extern crate mwparser_utils;

use std::str;
use std::process;
use std::io;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

use mfnf_export::*;
use mwparser_utils::CachedTexChecker;
use mediawiki_parser::transformations::TResult;


#[derive(Debug, StructOpt)]
#[structopt(name = "mfnf_ex", about = "This program renders an article syntax tree to a serial format (like LaTeX).")]
struct Args {
    /// Dump the default settings to stdout.
    #[structopt(short = "d", long = "dump-config")]
    dump_config: bool,
    /// Path to the input file.
    #[structopt(parse(from_os_str), short = "i", long = "input")]
    input_file: Option<PathBuf>,
    /// Path to the config file.
    #[structopt(parse(from_os_str), short = "c", long = "config")]
    config: Option<PathBuf>,
    /// Path to the texvccheck binary (formula checking).
    #[structopt(parse(from_os_str), short = "p", long = "texvccheck-path")]
    texvccheck_path: Option<PathBuf>,
    /// Base path for sections and media.
    #[structopt(parse(from_os_str), short = "b", long = "base-path")]
    base_path: Option<PathBuf>,
    /// Path to the article sections directory.
    #[structopt(parse(from_os_str), short = "s", long = "section-path")]
    section_path: Option<PathBuf>,
    /// Path to the media file directory.
    #[structopt(parse(from_os_str), short = "e", long = "media-path")]
    media_path: Option<PathBuf>,
    /// Path to article markers (includes / excludes).
    #[structopt(parse(from_os_str), short = "m", long = "markers")]
    marker_path: Option<PathBuf>,

    /// Title of the document.
    #[structopt(short = "t", long = "title")]
    doc_title: Option<String>,
    /// Revision of the document.
    #[structopt(short = "r", long = "revision")]
    doc_revision: Option<String>,

    /// The export target. (e.g. `latex` or `latex.print`)
    #[structopt()]
    target: String,

    /// Target-specific arguments.
    #[structopt()]
    target_args: Vec<String>,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::from_args();

    let general_settings = if let Some(path) = args.config {
        let file = fs::File::open(&path)?;
        serde_yaml::from_reader(&file)
            .expect("Error reading settings:")
    } else {
        GeneralSettings::default()
    };

    let mut settings = Settings::default();
    settings.general = general_settings;
    settings.runtime.target_name = args.target.clone();

    let orig_root: TResult;
    // section inclusion, etc. may fail, but deps shoud still be generated.
    let transformed_root: TResult;

    if let Some(title) = args.doc_title {
        settings.runtime.document_title = title
    }

    if let Some(revision) = args.doc_revision {
        settings.runtime.document_revision = revision
    }

    if let Some(base_path) = args.base_path {
        settings.general.base_path = base_path
    }

    if let Some(section_path) = args.section_path {
        settings.general.section_path = section_path
    }

    if let Some(media_path) = args.media_path {
        settings.general.media_path = media_path
    }

    if args.dump_config {
        println!("{}", serde_yaml::to_string(&settings.general)
            .expect("could not serialize default settings!"));
        process::exit(0);
    }

    if let Some(path) = args.marker_path {
        let file = fs::File::open(&path)?;
        settings.runtime.markers = serde_yaml::from_reader(&file)
            .expect("Error reading markers:")
    }

    if let Some(path) = args.texvccheck_path {
        settings.runtime.tex_checker = Some(CachedTexChecker::new(
            &path, 10_000
        ));
    } else {
        eprintln!("Warning: no texvccheck path, won't perform checks!");
    }

    let root = if let Some(path) = args.input_file {
        let file = fs::File::open(&path)?;
        serde_yaml::from_reader(&file)
    } else {
        serde_yaml::from_reader(io::stdin())
    }.expect("Error reading input:");

    orig_root = normalize(root, &settings);
    let root_clone = handle_transformation_result(&orig_root).clone();
    transformed_root = compose(root_clone, &settings);

    // export target
    let mut export_result = vec![];
    let target = match settings.general.targets.get(&args.target) {
        Some(t) => t.get_target(),
        None => {
            eprintln!("target not configured: {:?}", args.target);
            process::exit(1);
        }
    };

    let root = if target.include_sections() {
        handle_transformation_result(&transformed_root)
    } else {
        handle_transformation_result(&orig_root)
    };
    target.export(root, &settings, &args.target_args, &mut export_result)
        .expect("target export failed!");
    println!("{}", str::from_utf8(&export_result).unwrap());
    Ok(())
}

fn handle_transformation_result(result: &TResult) -> &mediawiki_parser::Element {

     match *result {
        Ok(ref e) => e,
        Err(ref e) => {
            eprintln!("{}", e);
            println!("{}", serde_yaml::to_string(&e)
                .expect("Could not serialize error!"));
            process::exit(1);
        }
    }
}
