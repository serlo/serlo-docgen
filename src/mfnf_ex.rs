//! CLI front end for the mfnf export tool.
//!
//! Applies some transformations to the input tree and exports it as defined by the given target.

extern crate mediawiki_parser;
extern crate serde_yaml;
extern crate serde_json;
extern crate argparse;
extern crate mfnf_export;

use std::str;
use std::process;
use std::io;
use std::fs;

use mfnf_export::*;
use mediawiki_parser::transformations::TResult;

use argparse::{ArgumentParser, StoreTrue, Store, Collect};


/// Program options and arguments
#[derive(Debug)]
struct Args {
    pub dump_config: bool,
    pub write_config_json: bool,
    pub input_file: String,
    pub config_file: String,
    pub doc_title: String,
    pub doc_revision: String,
    pub target: String,
    pub target_args: Vec<String>,
    pub texvccheck_path: String,
    pub sections_path: String,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            dump_config: false,
            write_config_json: false,
            input_file: String::new(),
            config_file: String::new(),
            doc_title: "<no document name specified>".to_string(),
            doc_revision: "latest".to_string(),
            target: String::new(),
            target_args: vec![],
            texvccheck_path: String::new(),
            sections_path: "sections".into(),
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
        ap.refer(&mut args.texvccheck_path).add_option(
            &["-p", "--texvccheck-path"],
            Store,
            "Path to the `texvccheck` executable."
        );
        ap.refer(&mut args.sections_path).add_option(
            &["-s", "--sections-path"],
            Store,
            "Path to the directory of included sections."
        );
        ap.refer(&mut args.target).add_argument(
            "target",
            Store,
            "The target to export, like `deps`, `sections`, `latex`, ..."
        );
        ap.refer(&mut args.target_args).add_argument(
            "args",
            Collect,
            "Additional arguments for a target. (e.g. wantet targets for `deps`)"
        );
        ap.parse_args_or_exit();
    }
    args
}

fn main() {
    let args = parse_args();

    let mut settings = if !args.config_file.is_empty() {
        let file = fs::File::open(&args.config_file)
            .expect("Could not open config file!");
        serde_yaml::from_reader(&file)
            .expect("Could not parse config file!")
    } else {
        Settings::default()
    };

    let orig_root: TResult;
    // section inclusion, etc. may fail, but deps shoud still be generated.
    let transformed_root: TResult;

    settings.document_title = args.doc_title.clone();
    settings.document_revision = args.doc_revision.clone();
    settings.section_path = args.sections_path;

    if args.dump_config {
        println!("{}", serde_yaml::to_string(&settings)
            .expect("could not serialize default settings!"));
        process::exit(0);
    }

    if args.texvccheck_path.is_empty() {
        eprintln!("Warning: no texvccheck path, won't perform checks!");
    } else {
        settings.texvccheck_path = args.texvccheck_path.clone();
        settings.check_tex_formulas = true;
    }

    let root = (if !args.input_file.is_empty() {
        let file = fs::File::open(&args.input_file)
            .expect("Could not open input file!");
        serde_yaml::from_reader(&file)
    } else {
        serde_yaml::from_reader(io::stdin())
    }).expect("Could not parse input!");

    orig_root = normalize(root, &settings);
    let root_clone = handle_transformation_result(&orig_root).clone();
    transformed_root = compose(root_clone, &settings);

    // export target
    let mut export_result = vec![];
    let target = match settings.targets.get(&args.target) {
        Some(t) => t.get_target(),
        None => {
            eprintln!("target not configured: {:?}", args.target);
            process::exit(1);
        }
    };
    let root = if target.do_include_sections() {
        handle_transformation_result(&transformed_root)
    } else {
        handle_transformation_result(&orig_root)
    };
    target.export(root, &settings, &args.target_args, &mut export_result)
        .expect("target export failed!");
    println!("{}", str::from_utf8(&export_result).unwrap());
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
