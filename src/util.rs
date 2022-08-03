//! Various utility functions and definitions.

use mediawiki_parser::*;
// re-export common util
use crate::meta::MediaMeta;
use crate::settings::Settings;
use crate::TargetType;
pub use mwparser_utils::{
    extract_plain_text, filename_to_make, path_methods, CachedTexChecker, TexChecker,
};
use serde_json;
use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::process;

pub const SECTION_INCLUSION_PREFIX: &str = "#lst:";

pub fn load_anchor_set(path: &str) -> io::Result<HashSet<String>> {
    let mut file = File::open(&path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<HashSet<String>>())
}

/// based on <https://github.com/bt/rust_urlencoding>
pub fn urlencode(data: &str) -> String {
    let mut escaped = String::new();
    for b in data.as_bytes().iter() {
        match *b as char {
            // Accepted characters
            'A'..='Z' | 'a'..='z' | '0'..='9' | '/' | ':' | '-' | '_' | '.' | '~' | '#' => {
                escaped.push(*b as char)
            }

            // Everything else is percent-encoded
            b => escaped.push_str(format!("%{:02X}", b as u32).as_str()),
        };
    }
    escaped
}

/// encode a url for mediawiki (underscore, urlencode)
pub fn mw_enc(input: &str) -> String {
    urlencode(&input.trim().replace(" ", "_"))
}

/// Checks if a internal reference target is available,
/// returns the anchor if found.
pub fn matching_anchor<'o>(target: &str, anchors: &'o HashSet<String>) -> Option<&'o String> {
    anchors.get(&mw_enc(target.trim().trim_start_matches(':')))
}

/// Returns a unicode character for a smiley description.
///
/// see also: <https://www.mediawiki.org/wiki/Template:Smiley>
pub fn smiley_to_unicode(input: &str) -> Option<char> {
    match input {
        ":)" | "smile" | ":-)" | ":-]" | "#default" => Some('\u{01F60A}'),
        ":(" | "sad" | "frown" | ":-(" | ":-[" => Some('\u{01F61E}'),
        "smirk" | "wink" | ";)" | ";-)" | ";-]" => Some('\u{01F60F}'),
        "grin" | ":-D" | ":D" | "lol" | "lach" => Some('\u{01F604}'),
        "surprise" | ":O" | ":-O" | "staun" => Some('\u{01F62E}'),
        "tongue" | ":P" | ":-P" => Some('\u{01F61B}'),
        "shades" | "cool" | "8-]" | "8)" | "8-)" => Some('\u{01F60E}'),
        "cry" | "wein" | ":'(" | ":-'(" => Some('\u{01F622}'),
        "devil-grin" | "devil" | "evil" | ">:-D" => Some('\u{01F608}'),
        "angry" | "wütend" | ">:(" | ">:[" => Some('\u{01F620}'),
        "confused" | "verwirrt" | "%)" | "%-)" | ":-/" => Some('\u{01F615}'),
        "confounded" | "very-confused" | ":-S" => Some('\u{01F616}'),
        "thumb" | "thumbsup" | "daumen" | "Daumen" => Some('\u{01F64C}'),
        "Facepalm" | "facepalm" => Some('\u{01F625}'),
        _ => None,
    }
}

/// Trim one pair of prefix and suffix from a string, ignoring input case.
pub fn trim_enclosing<'a>(input: &'a str, prefix: &str, suffix: &str) -> &'a str {
    let lower_input = input.to_lowercase();
    if lower_input.starts_with(prefix) && lower_input.ends_with(suffix) {
        return &input[prefix.len()..input.len() - suffix.len()];
    }
    input
}

/// Remove a prefix if found, ignoring input case.
pub fn trim_prefix<'a>(input: &'a str, prefix: &str) -> &'a str {
    let lower_input = input.to_lowercase();
    if lower_input.starts_with(prefix) {
        return &input[prefix.len()..];
    }
    input
}

/// Indent and trim a string.
pub fn indent_and_trim(input: &str, depth: usize, max_line_width: usize) -> String {
    const COMMENT_PREFIX: &str = "% ";

    let mut lines = vec![];
    for line in input.split('\n') {
        let trimmed = line.trim();
        let comment = trimmed.starts_with(COMMENT_PREFIX.trim());
        let line_depth = depth + line.len() - line.trim_start().len();
        let start_string = format!("{:depth$}", "", depth = line_depth);

        let mut new_line = start_string.clone();

        if trimmed.len() > max_line_width {
            for word in trimmed.split(' ') {
                let current_length = new_line.trim().len();

                if current_length + word.len() + 1 > max_line_width && current_length > 0 {
                    lines.push(new_line);
                    new_line = start_string.clone();
                    if comment {
                        new_line.push_str(COMMENT_PREFIX);
                    }
                }

                new_line.push_str(word);
                new_line.push(' ');
            }
            lines.push(new_line);
        } else {
            new_line.push_str(trimmed);
            lines.push(new_line);
        }
    }
    lines.join("\n")
}

struct TreeMatcher<'e, 'c> {
    pub result: bool,
    pub path: Vec<&'e Element>,
    pub predicate: &'c dyn Fn(&Element) -> bool,
}

impl<'e, 'c> Traversion<'e, ()> for TreeMatcher<'e, 'c> {
    path_methods!('e);

    fn work(&mut self, root: &Element, _: (), _: &mut dyn io::Write) -> io::Result<bool> {
        if (self.predicate)(root) {
            self.result = true;
            Ok(false)
        } else {
            Ok(true)
        }
    }
}

/// recursively tests a predicate for a AST.
pub fn tree_contains(tree: &Element, predicate: &dyn Fn(&Element) -> bool) -> bool {
    let mut matcher = TreeMatcher {
        result: false,
        path: vec![],
        predicate,
    };
    matcher
        .run(tree, (), &mut vec![])
        .expect("unexptected tree matcher IO error:");
    matcher.result
}

/// verifies a given "path" is only a plain filename without directory structure.
fn is_plain_file(path: &PathBuf) -> bool {
    let components = path.components();
    if components.count() != 1 {
        return false;
    }
    match path.components().next() {
        Some(c) => c.as_os_str() == path,
        None => false,
    }
}

pub fn iref_has_option(image: &InternalReference, options: &[&str]) -> bool {
    image
        .options
        .iter()
        .any(|ref o| options.contains(&extract_plain_text(o).to_lowercase().trim()))
}

/// Returns wether an image is semantically a thumbnail image.
pub fn is_thumb(image: &InternalReference) -> bool {
    iref_has_option(image, &["thumbnail", "thumb", "miniatur", "mini"])
}

/// Returns wether an image is semantically a centered image.
pub fn is_centered(image: &InternalReference) -> bool {
    iref_has_option(image, &["center", "zentriert"])
}

/// Path of a section file.
pub fn get_section_path(article: &str, section: &str, section_path: &PathBuf) -> String {
    if !is_plain_file(&PathBuf::from(article)) {
        eprintln!(
            "article name \"{}\" contains path elements. \
             This could be dangerous! Abort.",
            article
        );
        process::exit(1);
    }

    if !is_plain_file(&PathBuf::from(section)) {
        eprintln!(
            "section name \"{}\" contains path elements. \
             This could be dangerous! Abort.",
            section
        );
        process::exit(1);
    }

    let article = filename_to_make(&article);
    let section = filename_to_make(&section);
    let path = section_path
        .join(&article)
        .join(&section)
        .join("latest")
        .with_extension("json");
    path.to_string_lossy().to_string()
}

/// This object can be rendered by a traversion with the unit type as settings.
pub trait Renderable {
    fn render<'e>(&'e self, renderer: &mut dyn Traversion<'e, ()>) -> io::Result<String>;
}

impl Renderable for Element {
    fn render<'e>(&'e self, renderer: &mut dyn Traversion<'e, ()>) -> io::Result<String> {
        let mut temp = vec![];
        renderer.run(self, (), &mut temp)?;
        Ok(String::from_utf8(temp).unwrap())
    }
}

impl Renderable for [Element] {
    fn render<'e>(&'e self, renderer: &mut dyn Traversion<'e, ()>) -> io::Result<String> {
        let mut temp = vec![];
        renderer.run_vec(self, (), &mut temp)?;
        Ok(String::from_utf8(temp).unwrap())
    }
}

/// Extract all child nodes from an elment in a list.
/// If an element has multiple fields, they are concatenated
/// in a semantically useful order.
pub fn extract_content(root: Element) -> Option<Vec<Element>> {
    match root {
        Element::Document(e) => Some(e.content),
        Element::Formatted(e) => Some(e.content),
        Element::Paragraph(e) => Some(e.content),
        Element::ListItem(e) => Some(e.content),
        Element::List(e) => Some(e.content),
        Element::TableCell(e) => Some(e.content),
        Element::HtmlTag(e) => Some(e.content),
        Element::Gallery(e) => Some(e.content),
        Element::Heading(mut e) => {
            e.caption.append(&mut e.content);
            Some(e.caption)
        }
        Element::Template(mut e) => {
            e.name.append(&mut e.content);
            Some(e.name)
        }
        Element::TemplateArgument(e) => Some(e.value),
        Element::InternalReference(mut e) => {
            for mut option in &mut e.options {
                e.target.append(&mut option);
            }
            e.target.append(&mut e.caption);
            Some(e.target)
        }
        Element::ExternalReference(e) => Some(e.caption),
        Element::Table(mut e) => {
            e.caption.append(&mut e.rows);
            Some(e.caption)
        }
        Element::TableRow(e) => Some(e.cells),
        Element::Text(_) | Element::Comment(_) | Element::Error(_) => None,
    }
}

/// loads media meta data from the corresponding .meta file,
/// panics on error.
pub fn load_media_meta(name: &[Element], settings: &Settings) -> MediaMeta {
    let mut file_path = build_media_path(name, settings);
    let mut filename = file_path
        .file_name()
        .unwrap_or_else(|| panic!("no file component in {:?}!", &file_path))
        .to_os_string();
    filename.push(".meta");
    file_path.set_file_name(filename);

    let file =
        File::open(&file_path).unwrap_or_else(|_| panic!("could not open {:?}!", &file_path));
    serde_json::from_reader(&file)
        .unwrap_or_else(|_| panic!("could not deserialize {:?}!", &file_path))
}

/// Get the target-specific version of a file extension.
pub fn map_extension(target: TargetType, extension: &str) -> Option<String> {
    match target {
        TargetType::SectionDeps => None,
        TargetType::MediaDeps => None,
        TargetType::Sections => None,
        TargetType::Normalize => None,
        TargetType::Compose => None,
        TargetType::Anchors => None,
        TargetType::Latex => Some(
            match extension.trim().to_lowercase().as_str() {
                "png" => "%.pdf",
                "svg" => "%.pdf",
                "eps" => "%.pdf",
                "jpg" => "%.pdf",
                "jpeg" => "%.pdf",
                "gif" => "%.qr.pdf",
                "webm" => "%.qr.pdf",
                "mp4" => "%.qr.pdf",
                "pdf" => "plain.%",
                _ => return None,
            }
            .replace("%", &extension),
        ),
        TargetType::PDF => None,
        TargetType::Stats => Some("dummy".to_string()),
        TargetType::HTML => Some(extension.to_string()),
        TargetType::Serlo => None,
        TargetType::Formula => None,
    }
}

pub fn mapped_media_path(target: TargetType, name: &[Element], settings: &Settings) -> PathBuf {
    let file_path = build_media_path(name, settings);
    let ext = file_path
        .extension()
        .expect("media file has no file extension!");

    let target_extension = map_extension(target, &ext.to_string_lossy())
        .expect("target {:?} does define media extensions!");

    file_path.with_extension(&target_extension)
}

pub fn build_media_path(name: &[Element], settings: &Settings) -> PathBuf {
    let name_str = extract_plain_text(name);
    let mut trimmed = name_str.trim();

    for prefix in &settings.file_prefixes {
        trimmed = trim_prefix(trimmed, prefix);
    }

    let name_path = filename_to_make(trimmed.trim());
    PathBuf::from(&settings.media_path).join(&name_path)
}

pub fn is_file(iref: &InternalReference, settings: &Settings) -> bool {
    let plain = extract_plain_text(&iref.target).trim().to_lowercase();
    settings.file_prefixes.iter().any(|p| plain.starts_with(p))
}
