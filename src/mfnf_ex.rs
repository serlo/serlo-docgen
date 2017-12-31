extern crate mediawiki_parser;
extern crate serde_yaml;
extern crate argparse;
extern crate mfnf_export;
extern crate toml;

use std::str;
use std::process;
use mfnf_export::settings::*;
use mfnf_export::{latex, deps};

use mediawiki_parser::util::{read_file, read_stdin};
use argparse::{ArgumentParser, StoreTrue, Store, Collect};


/// Program options and arguments
#[derive(Debug)]
struct Args {
    pub use_stdin: bool,
    pub dump_config: bool,
    pub input_file: String,
    pub config_file: String,
    pub doc_title: String,
    pub targets: Vec<String>,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            use_stdin: false,
            dump_config: false,
            input_file: String::new(),
            config_file: String::new(),
            doc_title: "<no document name specified>".to_string(),
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
        ap.refer(&mut args.use_stdin).add_option(
            &["-s", "--stdin"],
            StoreTrue,
            "Use stdin as input file",
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

    for target_name in &args.targets {
        match &target_name[..] {
            "latex" => {
                result.push(Target {
                    name: target_name.to_string(),
                    output_path: "./export/latex/".to_string(),
                    settings: settings.clone(),
                    export_func: latex::export_article,
                });
            },
            "deps" => {
                result.push(Target {
                    name: target_name.to_string(),
                    output_path: "./export/deps/".to_string(),
                    settings: settings.clone(),
                    export_func: deps::collect_article_deps,
                });
            },
            _ => {
                eprintln!("unsupported target: `{}`", target_name);
            }
        };
    }
    result
}

fn main() {

    let result: mediawiki_parser::transformations::TResult;
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

    let input = if args.use_stdin {
        read_stdin()
    } else if !args.input_file.is_empty() {
        read_file(&args.input_file)
    } else {
        eprintln!("No input source specified!");
        process::exit(1);
    };

    let root = serde_yaml::from_str(&input)
        .expect("Could not parse input file!");

    let mut path = vec![];
    result = mfnf_export::apply_transformations(root, general_settings);

    match result {
        Ok(ref e) => {
            for target in &targets[..] {
                let mut result = vec![];
                (target.export_func)(&e, &mut path, &target.settings, &mut result)
                .expect("Could not output export!");
                println!("{}", str::from_utf8(&result).unwrap());
            };
        },
        Err(ref e) => {
            eprintln!("{}", e);
            println!("{}", serde_yaml::to_string(&e)
                .expect("Could not serialize error!"));
        }
    }
}
