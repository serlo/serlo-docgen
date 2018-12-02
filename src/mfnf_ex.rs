//! CLI front end for the mfnf export tool.
//!
//! Applies some transformations to the input tree and exports it as defined by the given target.

extern crate mediawiki_parser;
extern crate mfnf_export;
extern crate mwparser_utils;
extern crate serde_json;
extern crate serde_yaml;
extern crate structopt;

use mediawiki_parser::Element;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::str;
use structopt::StructOpt;

use mfnf_export::*;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mfnf_ex",
    about = "This program renders an article syntax tree to a serial format (like LaTeX, HTML, ...)."
)]
struct Args {
    /// Path to the input file.
    #[structopt(parse(from_os_str), short = "i", long = "input")]
    input_file: Option<PathBuf>,
    /// Path to the config file.
    #[structopt(parse(from_os_str), short = "c", long = "config-file")]
    config_file: Option<PathBuf>,
    /// Path to the media file directory.
    #[structopt(parse(from_os_str), short = "e", long = "media-path")]
    media_path: Option<PathBuf>,

    /// The target configuration (subtarget) to use. e.g. `default` or `print`.
    configuration: String,

    #[structopt(subcommand)]
    cmd: Commands,
}

/// Target subcommands
#[derive(Debug, StructOpt)]
enum Commands {
    #[structopt(name = "normalize", about = "normalize the input article.")]
    Normalize(NormalizeArgs),
    #[structopt(name = "compose", about = "compose the input article.")]
    Compose(ComposeArgs),
    #[structopt(
        name = "section-deps",
        about = "generate a makefile declaring included sections as prerequisites of `base_file`."
    )]
    SectionDeps(SectionDepArgs),
    #[structopt(
        name = "media-deps",
        about = "generate a makefile declaring included media as prerequisites of `base_file`."
    )]
    MediaDeps(MediaDepArgs),
    #[structopt(
        name = "sections",
        about = "extract a section from a document."
    )]
    Sections(SectionsArgs),
    #[structopt(
        name = "anchors",
        about = "export a list of anchor targets for this document."
    )]
    Anchors(AnchorsArgs),
    #[structopt(name = "html", about = "export the document as html.")]
    HTML(HTMLArgs),
    #[structopt(name = "latex", about = "export the document as latex.")]
    Latex(LatexArgs),
    #[structopt(name = "pdf", about = "export pdf options for the document.")]
    PDF(PDFArgs),
    #[structopt(name = "stats", about = "export document statistics.")]
    Stats(StatsArgs),
    #[structopt(
        name = "dump-config",
        about = "dump the current configuration to stdout."
    )]
    DumpConfig,
}

macro_rules! find_target {
    ($var:path, $settings:ident, $args:ident) => {{
        if let Some(Some(t)) = $settings
            .general
            .targets
            .get(&$args.configuration)
            .map(|targets| {
                targets
                    .iter()
                    .find_map(|c| if let $var(t) = c { Some(t) } else { None })
            }) {
            t
        } else {
            panic!(
                "target not found in configuration \"{}\"!",
                &$args.configuration
            );
        }
    }};
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::from_args();

    let mut settings = Settings::default();
    settings.general = if let Some(path) = args.config_file {
        let file = fs::File::open(&path)?;
        serde_yaml::from_reader(&file).expect("Error reading settings:")
    } else {
        GeneralSettings::default()
    };

    if let Some(media_path) = args.media_path {
        settings.general.media_path = media_path
    }

    /*
    if let Some(path) = args.available_anchors {
        let mut file = fs::File::open(&path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        settings.runtime.available_anchors = content
            .split("\n")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<HashSet<String>>();
    }*/

    let root: Element = if let Some(path) = args.input_file {
        let file = fs::File::open(&path)?;
        serde_json::from_reader(&file).expect("error reading input!")
    } else {
        serde_json::from_reader(io::stdin()).expect("error reading input!")
    };

    match &args.cmd {
        Commands::DumpConfig => println!(
            "{}",
            serde_yaml::to_string(&settings.general)
                .expect("could not serialize default settings!")
        ),
        Commands::Anchors(ref target_args) => find_target!(Targets::Anchors, settings, args)
            .export(&root, (), target_args, &mut io::stdout())?,
        Commands::Sections(ref target_args) => find_target!(Targets::Sections, settings, args)
            .export(&root, (), target_args, &mut io::stdout())?,
        Commands::SectionDeps(ref target_args) => find_target!(
            Targets::SectionDeps,
            settings,
            args
        ).export(&root, (), target_args, &mut io::stdout())?,
        Commands::MediaDeps(ref target_args) => find_target!(Targets::MediaDeps, settings, args)
            .export(&root, &settings, target_args, &mut io::stdout())?,
        Commands::Normalize(ref target_args) => find_target!(Targets::Normalize, settings, args)
            .export(&root, &settings, target_args, &mut io::stdout())?,
        Commands::Compose(ref target_args) => find_target!(Targets::Compose, settings, args)
            .export(&root, (), target_args, &mut io::stdout())?,
        Commands::Latex(ref target_args) => find_target!(Targets::Latex, settings, args).export(
            &root,
            &settings,
            target_args,
            &mut io::stdout(),
        )?,
        Commands::PDF(ref target_args) => find_target!(Targets::PDF, settings, args).export(
            &root,
            &settings,
            target_args,
            &mut io::stdout(),
        )?,
        Commands::Stats(ref target_args) => find_target!(Targets::Stats, settings, args).export(
            &root,
            &settings,
            target_args,
            &mut io::stdout(),
        )?,
        Commands::HTML(ref target_args) => find_target!(Targets::HTML, settings, args).export(
            &root,
            &settings,
            target_args,
            &mut io::stdout(),
        )?,
    }
    Ok(())
}
