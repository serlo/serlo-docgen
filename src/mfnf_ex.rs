//! CLI front end for the mfnf export tool.
//!
//! Applies some transformations to the input tree and exports it as defined by the given target.

extern crate mediawiki_parser;
extern crate serde_yaml;
extern crate argparse;
extern crate mfnf_export;

use std::str;
use std::process;
use std::io;
use std::fs;

use mfnf_export::*;

use argparse::{ArgumentParser, StoreTrue, Store, Collect};


/// Program options and arguments
#[derive(Debug)]
struct Args {
    pub dump_config: bool,
    pub input_file: String,
    pub config_file: String,
    pub doc_title: String,
    pub doc_revision: String,
    pub targets: Vec<String>,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            dump_config: false,
            input_file: String::new(),
            config_file: String::new(),
            doc_title: "<no document name specified>".to_string(),
            doc_revision: "latest".to_string(),
            targets: vec![],
        }
    }
}

fn parse_args() -> Args {
    let mut args = Args::default();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description(
            "This program applies transformations specific to the \
                \"Mathe für nicht-Freaks\"-Project to a syntax tree."
        );
        ap.refer(&mut args.input_file).add_option(
            &["-i", "--input"],
            Store,
            "Path to the input file",
        );
        ap.refer(&mut args.doc_title).add_option(
            &["-t",  "--title"],
            Store,
            "Title of the input document",
        );
        ap.refer(&mut args.doc_revision).add_option(
            &["-r", "--revision"],
            Store,
            "Revision ID of the input document"
        );
        ap.refer(&mut args.dump_config).add_option(
            &["-d", "--dump-settings"],
            StoreTrue,
            "Dump the default settings to stdout."
        );
        ap.refer(&mut args.config_file).add_option(
            &["-c", "--config"],
            Store,
            "A config file to override the default options."
        );
        ap.refer(&mut args.targets).add_argument(
            "targets",
            Collect,
            "List of targets to export. Currently supported: `latex`"
        );
        ap.parse_args_or_exit();
    }
    args
}

fn main() {
    let args = parse_args();
    let mut settings = settings::Settings::default();

    let orig_root: mediawiki_parser::transformations::TResult;
    // section inclusion, etc. may fail, but deps shoud still be generated.
    let transformed_root: mediawiki_parser::transformations::TResult;

    /*
    if !args.config_file.is_empty() {
        settings.merge(config::File::with_name(&args.config_file))
            .expect("Could not parse settings file!");
    };
    */

    settings.document_title = args.doc_title.clone();
    settings.document_revision = args.doc_revision.clone();

    /*
    if args.dump_config {
        println!("{}", DEFAULT_SETTINGS);
        process::exit(0);
    }
    */

    if args.targets.is_empty() {
        eprintln!("No target specified!");
        process::exit(1);
    }

    let root = (if !args.input_file.is_empty() {
        let file = fs::File::open(&args.input_file)
            .expect("Could not open input file!");
        serde_yaml::from_reader(&file)
    } else {
        serde_yaml::from_reader(io::stdin())
    }).expect("Could not parse input!");

    orig_root = mfnf_export::apply_universal_transformations(root, &settings);
    let root_clone = handle_transformation_result(&orig_root).clone();
    transformed_root = mfnf_export::apply_output_transformations(root_clone, &settings);

    for target in &args.targets {
        let mut export_result = vec![];
        let target = match settings.targets.get(target) {
            Some(t) => t.get_target(),
            None => {
                eprintln!("target not configured: {:?}", target);
                continue
            }
        };
        let root = if target.do_include_sections() {
            handle_transformation_result(&transformed_root)
        } else {
            handle_transformation_result(&orig_root)
        };
        target.export(root, &settings, &mut export_result)
            .expect("target export failed!");
        println!("{}", str::from_utf8(&export_result).unwrap());
    }
}

fn handle_transformation_result(result: &mediawiki_parser::transformations::TResult)
    -> &mediawiki_parser::ast::Element {

     match result {
        &Ok(ref e) => return e,
        &Err(ref e) => {
            eprintln!("{}", e);
            println!("{}", serde_yaml::to_string(&e)
                .expect("Could not serialize error!"));
            process::exit(1);
        }
    }
}
