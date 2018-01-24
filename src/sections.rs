//! Implements the `sections` target which writes out parts of the syntax tree.
//!
//! This target operates on the same syntax tree as the `deps` target. It extracts
//! parts of the document tree marked by `<section />` tags and writes them to a
//! directory specified through the transformation settings in the YAML format.

use std::io;
use std::str;
use std::collections::HashMap;
use settings::*;
use mediawiki_parser::ast::*;
use mediawiki_parser::transformations::*;
use util::*;
use std::path;
use std::fs::File;
use std::io::Write;
use std::fs::DirBuilder;
use serde_yaml;


/// Write marked document section to the filesystem.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SectionsTarget {
    pub extension_mapping: HashMap<String, String>,
}

impl Target for SectionsTarget {
    fn get_name(&self) -> &str { "sections" }
    fn do_include_sections(&self) -> bool { false }
    fn do_generate_dependencies(&self) -> bool { false }
    fn get_target_extension(&self) -> &str { "yml" }
    fn get_extension_mapping(&self) -> &HashMap<String, String> {
        &self.extension_mapping
    }
    fn export<'a>(&self,
                root: &'a Element,
                path: &mut Vec<&'a Element>,
                settings: &Settings,
                out: &mut io::Write) -> io::Result<()> {

        let mut name_collector = SectionNameCollector::default();
        name_collector.run(root, settings, &mut vec![])?;

        for section in name_collector.sections {
            if section.is_empty() {
                continue
            }

            let mut start_finder = SectionFinder::new(&section, true);
            let mut end_finder = SectionFinder::new(&section, false);

            start_finder.run(root, (), &mut vec![])?;
            end_finder.run(root, (), &mut vec![])?;

            if start_finder.result.is_empty() || end_finder.result.is_empty() {
                continue
            }

            let inter = get_intermediary(&start_finder.result, &end_finder.result);

            let mut filename = settings.document_revision.clone();
            let file_ext = &settings.section_ext;
            let section_path = &settings.section_path;
            let doctitle = &settings.document_title;

            filename.push('.');
            filename.push_str(&file_ext);

            let mut path = path::Path::new(&section_path)
                .join(&filename_to_make(&doctitle))
                .join(&filename_to_make(&section.clone()));

            DirBuilder::new()
                .recursive(true)
                .create(&path)?;

            path = path.join(&filename);

            let mut file = File::create(&path)?;
            file.write_all(&serde_yaml::to_string(&inter)
                .expect("could not serialize section!")
                .as_bytes())?;

        }
        Ok(())
    }
}

/// Paramters for section filtering transformation.
#[derive(Debug, Clone)]
struct SectionFilter<'a, 'b: 'a> {
    pub begin: &'a Vec<&'b Element>,
    pub end: &'a Vec<&'b Element>,
    pub include_pre: bool,
}


/// Recursively trim a subtree to only contain the elements
/// enclosed by the section paths in SectionFilter.
fn filter_section_element(root: &Element,
                          path: &Vec<&Element>,
                          settings: &SectionFilter) -> TResult {

    recurse_clone_template(&filter_section_element,
                           root, path, settings,
                           &filter_section_subtree)
}


/// Recursively trim a list of elments to only contain the elements
/// enclosed by the section paths in SectionFilter.
fn filter_section_subtree<'a>(_func: &TFunc<&SectionFilter>,
                              content: &Vec<Element>,
                              path: &Vec<&'a Element>,
                              settings: &SectionFilter) -> TListResult {
    let mut result = vec![];
    let mut found_begin = false;

    for child in content {
        if settings.begin.contains(&child) {

            found_begin = true;

            // ignore the starting section tag
            if !(Some(&child) == settings.begin.last()) {
                result.push(filter_section_element(&child, path, &settings.clone())
                    .expect("error in section filter"));
            }
            continue;
        }
        if settings.end.contains(&child) {

            let mut child_settings = settings.clone();
            child_settings.include_pre = true;

            // ignore the ending section tag
            if !(Some(&child) == settings.end.last()) {
                result.push(filter_section_element(&child, path, &child_settings)
                    .expect("error in section filter"));
            }
            break;
        }

        if found_begin || settings.include_pre {
            result.push(child.clone());
        }

    }
    Ok(result)
}

/// Get the children of the lowest common element of two section paths.
/// Child nodes before and after the section tag are discarded.
fn get_intermediary<'a>(start: &Vec<&'a Element>, end: &Vec<&'a Element>) -> Vec<Element> {
    // lowest common node
    let mut common = None;
    for ps in start.iter().rev() {
        for pe in end.iter().rev() {
            if pe == ps {
                common = Some(ps);
                break;
            }
        }
        if let Some(_) = common {
            break;
        }
    }
    if let None = common {
        return vec![];
    }
    let common = common.unwrap();
    let section_filter = SectionFilter { begin: start, end, include_pre: false };
    let filtered = filter_section_element(&common, &vec![], &section_filter)
        .expect("error in section filter");
    extract_content(filtered).unwrap_or(vec![])
}

/// Collect the names of all beginning sections in a document.
#[derive(Default)]
struct SectionNameCollector<'e> {
    path: Vec<&'e Element>,
    pub sections: Vec<String>,
}

impl<'e, 's: 'e> Traversion<'e, &'s Settings> for SectionNameCollector<'e> {
    fn path_push(&mut self, root: &'e Element) {
        self.path.push(root);
    }
    fn path_pop(&mut self) -> Option<&'e Element> {
        self.path.pop()
    }
    fn get_path(&self) -> &Vec<&'e Element> {
        &self.path
    }
    fn work(&mut self,
            root: &'e Element,
            settings: &'s Settings,
            out: &mut io::Write) -> io::Result<bool> {

        if let &Element::HtmlTag { ref name, ref attributes, .. } = root {
            if name.to_lowercase() == "section"  {
                for attr in attributes {
                    if attr.key == "begin" {
                        self.sections.push(attr.value.trim().into());
                    }
                }
            }
        };
        Ok(true)
    }
}

/// Return a path to the start / end of a section
#[derive(Default)]
struct SectionFinder<'e, 'a> {
    /// label of the section to find.
    pub label: &'a str,
    /// get start or end of section?
    pub begin: bool,
    path: Vec<&'e Element>,
    /// the resulting path.
    pub result: Vec<&'e Element>
}

impl<'e, 'a> Traversion<'e, ()> for SectionFinder<'e, 'a> {
    fn path_push(&mut self, root: &'e Element) {
        self.path.push(root);
    }
    fn path_pop(&mut self) -> Option<&'e Element> {
        self.path.pop()
    }
    fn get_path(&self) -> &Vec<&'e Element> {
        &self.path
    }
    fn work(&mut self,
            root: &'e Element,
            settings: (),
            out: &mut io::Write) -> io::Result<bool> {

        // end recursion if result is found
        if self.result.len() > 0 {
            return Ok(false)
        }

        if let &Element::HtmlTag { ref name, ref attributes, .. } = root {
            if name.to_lowercase() == "section" {
                for attr in attributes {
                    if attr.key.to_lowercase() == if self.begin {"begin"} else {"end"}
                        && attr.value.to_lowercase() == self.label.to_lowercase() {
                        self.result = self.path.clone();
                    }
                }
            }
        };
        Ok(true)
    }
}

impl<'a, 'e> SectionFinder<'e, 'a> {
    fn new(label: &'a str, begin: bool) -> SectionFinder {
        SectionFinder {
            label,
            begin,
            path: vec![],
            result: vec![],
        }
    }
}

