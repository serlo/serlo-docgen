extern crate mediawiki_parser;
extern crate serde_yaml;
extern crate argparse;
extern crate mfnf_transformations;
extern crate toml;

use std::process;
use argparse::{ArgumentParser, StoreTrue, Store};


fn main() {
    let mut use_stdin = false;
    let mut dump_config = false;
    let mut input_file = "".to_string();
    let mut config_file = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description(
            "This program applies transformations specific to the \
             \"Mathe für nicht-Freaks\"-Project to a syntax tree."
        );
        ap.refer(&mut use_stdin).add_option(
            &["-s", "--stdin"],
            StoreTrue,
            "Use stdin as input file",
        );
        ap.refer(&mut input_file).add_option(
            &["-i", "--input"],
            Store,
            "Path to the input file",
        );
        ap.refer(&mut dump_config).add_option(
            &["-d", "--dump-settings"],
            StoreTrue,
            "Dump the default settings to stdout."
        );
        ap.refer(&mut config_file).add_option(
            &["-c", "--config"],
            Store,
            "A config file to override the default options."
        );
        ap.parse_args_or_exit();
    }
    let config = if config_file.is_empty() {
        mfnf_transformations::settings::Settings::default()
    } else {
        let config_source = mediawiki_parser::util::read_file(&config_file);
        toml::from_str(&config_source)
            .expect("Could not parse settings file!")
    };

    if dump_config {
        println!("{}", toml::to_string(&config)
            .expect("Could serialize settings!"));
        process::exit(0);
    }

    let input: String;
    if use_stdin {
        input = mediawiki_parser::util::read_stdin();
    } else if !input_file.is_empty() {
        input = mediawiki_parser::util::read_file(&input_file);
    } else {
        eprintln!("No input source specified!");
        process::exit(1);
    }


    let root = serde_yaml::from_str(&input).expect("Could not parse input file!");
    let result = mfnf_transformations::apply_transformations(root, &config);
    match result {
        Ok(e) => {
            println!("{}", serde_yaml::to_string(&e).expect("Could not serialize output!"));
        },
        Err(e) => {
            eprintln!("{}", e);
            println!("{}", serde_yaml::to_string(&e).expect("Could not serialize error!"));
        }
    };
}
