//! CLI front end for the mfnf export tool.
//!
//! Applies some transformations to the input tree and exports it as defined by the given target.

extern crate mediawiki_parser;
extern crate serde_yaml;
extern crate argparse;
extern crate mfnf_export;
extern crate toml;

use std::str;
use std::process;
use std::io;
use std::fs;
use mfnf_export::settings::*;
use mfnf_export::{latex, deps, sections};

use mediawiki_parser::util::{read_file};
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

fn build_targets(args: &Args) -> Vec<Target> {
    let mut result = vec![];

    let mut settings = if args.config_file.is_empty() {
        Settings::default()
    } else {
        let config_source = read_file(&args.config_file);
        toml::from_str(&config_source)
            .expect("Could not parse settings file!")
    };

    settings.document_title = args.doc_title.clone();
    settings.document_revision = args.doc_revision.clone();

    for target_name in &args.targets {
        match &target_name[..] {
            "latex" => {
                result.push(Target {
                    name: target_name.to_string(),
                    output_path: "./export/latex/".to_string(),
                    settings: settings.clone(),
                    export_func: &latex::export_article,
                    with_transformation: true,
                });
            },
            "deps" => {
                result.push(Target {
                    name: target_name.to_string(),
                    output_path: "./export/deps/".to_string(),
                    settings: settings.clone(),
                    export_func: &deps::export_article_deps,
                    with_transformation: false,
                });
            },
            "sections" => {
                result.push(Target {
                    name: target_name.to_string(),
                    output_path: "./export/sections/".to_string(),
                    settings:  settings.clone(),
                    export_func: &sections::collect_sections,
                    with_transformation: false,
                });
            }
            _ => {
                eprintln!("unsupported target: `{}`", target_name);
            }
        };
    }
    result
}

fn main() {
    let general_root: mediawiki_parser::ast::Element;
    let transformed_root: mediawiki_parser::ast::Element;
    let args = parse_args();
    let targets = build_targets(&args);

    if targets.is_empty() {
        eprintln!("No target specified!");
        process::exit(1);
    }

    let general_settings = &targets.first().unwrap().settings;

    if args.dump_config {
        println!("{}", serde_yaml::to_string(general_settings)
            .expect("Could serialize settings!"));
        process::exit(0);
    }

    let root = (if !args.input_file.is_empty() {
        let file = fs::File::open(&args.input_file)
            .expect("Could not open input file!");
        serde_yaml::from_reader(&file)
    } else {
        serde_yaml::from_reader(io::stdin())
    }).expect("Could not parse input!");

    let temp_root = mfnf_export::apply_universal_transformations(root, general_settings);
    general_root = handle_transformation_result(temp_root);
    let temp_root = mfnf_export::apply_output_transformations(general_root.clone(), general_settings);
    transformed_root = handle_transformation_result(temp_root);

    for target in &targets[..] {
        let mut path = vec![];
        let mut export_result = vec![];
        (target.export_func)(
            // pull dependencies from original tree
            if target.with_transformation {
                &transformed_root
            } else {
                &general_root
            },
            &mut path,
            &target.settings,
            &mut export_result
        ).expect("could not serialize target!");
        println!("{}", str::from_utf8(&export_result).unwrap());
    }
}

fn handle_transformation_result(result: mediawiki_parser::transformations::TResult) -> mediawiki_parser::ast::Element {
    match result {
        Ok(e) => return e,
        Err(ref e) => {
            eprintln!("{}", e);
            println!("{}", serde_yaml::to_string(&e)
                .expect("Could not serialize error!"));
            process::exit(1);
        }
    }
}
