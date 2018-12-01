//! CLI front end for the mfnf export tool.
//!
//! Applies some transformations to the input tree and exports it as defined by the given target.

extern crate mediawiki_parser;
extern crate mfnf_export;
extern crate mwparser_utils;
extern crate serde_json;
extern crate serde_yaml;
extern crate structopt;

use std::collections::HashSet;
use std::fs;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::process;
use std::str;
use structopt::StructOpt;

use mfnf_export::*;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mfnf_ex",
    about = "This program renders an article syntax tree to a serial format (like LaTeX, HTML, ...)."
)]
struct Args {
    /// Dump the default settings to stdout.
    #[structopt(long = "dump-config")]
    dump_config: bool,
    /// List the available targets.
    #[structopt(long = "list-targets")]
    list_targets: bool,

    /// Path to the input file.
    #[structopt(parse(from_os_str), short = "i", long = "input")]
    input_file: Option<PathBuf>,
    /// Path to the config file.
    #[structopt(parse(from_os_str), short = "c", long = "config")]
    config: Option<PathBuf>,
    /// Path to the media file directory.
    #[structopt(parse(from_os_str), short = "e", long = "media-path")]
    media_path: Option<PathBuf>,
    /// Path to a list of link targets (anchors) available in the export.
    #[structopt(parse(from_os_str), short = "a", long = "available-anchors")]
    available_anchors: Option<PathBuf>,

    /// Title of the document.
    #[structopt(short = "t", long = "title")]
    doc_title: Option<String>,
    /// Revision of the document.
    #[structopt(short = "r", long = "revision")]
    doc_revision: Option<String>,

    /// The export target. (e.g. `latex` or `html.print`)
    #[structopt()]
    target: String,

    /// Target-specific arguments. (Use `target` --help for more info)
    #[structopt()]
    target_args: Vec<String>,
}

fn main() -> Result<(), std::io::Error> {
    let mut args = Args::from_args();

    let general_settings = if let Some(path) = args.config {
        let file = fs::File::open(&path)?;
        serde_yaml::from_reader(&file).expect("Error reading settings:")
    } else {
        GeneralSettings::default()
    };

    let mut settings = Settings::default();
    settings.general = general_settings;
    settings.runtime.target_name = args.target.clone();

    if let Some(title) = args.doc_title {
        settings.runtime.document_title = title
    }

    if let Some(revision) = args.doc_revision {
        settings.runtime.document_revision = revision
    }

    if let Some(media_path) = args.media_path {
        settings.general.media_path = media_path
    }

    if args.dump_config {
        println!(
            "{}",
            serde_yaml::to_string(&settings.general)
                .expect("could not serialize default settings!")
        );
        process::exit(0);
    }

    if args.list_targets {
        let targets = settings
            .general
            .targets
            .iter()
            .map(|t| t.0.to_string())
            .collect::<Vec<String>>();

        println!("{}", targets.join(", "));
        process::exit(0);
    }

    if let Some(path) = args.available_anchors {
        let mut file = fs::File::open(&path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        settings.runtime.available_anchors = content
            .split("\n")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<HashSet<String>>();
    }

    let root = if let Some(path) = args.input_file {
        let file = fs::File::open(&path)?;
        serde_json::from_reader(&file).expect("error reading input!")
    } else {
        serde_json::from_reader(io::stdin()).expect("error reading input!")
    };

    // export target
    let mut export_result = vec![];
    let target = match settings.general.targets.get(&args.target) {
        Some(t) => t.get_target(),
        None => {
            eprintln!("target not configured: {:?}", args.target);
            process::exit(1);
        }
    };
    args.target_args.insert(0, args.target.clone());
    target
        .export(&root, &settings, &args.target_args, &mut export_result)
        .expect("target export failed!");
    println!("{}", str::from_utf8(&export_result).unwrap());
    Ok(())
}
