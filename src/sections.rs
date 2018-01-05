use std::io;
use std::str;
use settings::Settings;
use mediawiki_parser::ast::*;
use mediawiki_parser::transformations::*;
use util::*;
use serde_yaml;

/// Metadata structure for document sections.
#[derive(Serialize, Deserialize, Debug)]
pub struct Section {
    pub title: String,
    pub tree: Vec<Element>,
    pub position: Span,
}

/// Write all section names encountered to an output file.
pub fn collect_section_names<'a>(root: &'a Element,
                                 path: &mut Vec<&'a Element>,
                                 settings: &Settings,
                                 out: &mut io::Write) -> io::Result<()> {

    if let &Element::HtmlTag { ref name, ref attributes, .. } = root {
        if name.to_lowercase() == "section"  {
            for attr in attributes {
                if attr.key == "begin" {
                    writeln!(out, "{}", attr.value)?;
                }
            }
        }
    };
    traverse_with(&collect_section_names, root, path, settings, out)?;
    Ok(())
}

/// Collect all sections in a file and serialize them to stdout.
pub fn collect_sections<'a>(root: &'a Element,
                            _path: &mut Vec<&'a Element>,
                            settings: &Settings,
                            out: &mut io::Write) -> io::Result<()> {

    let mut sections_string = vec![];
    collect_section_names(root, &mut vec![], settings, &mut sections_string)?;
    let sections_string = String::from_utf8(sections_string).unwrap();

    // list of section names defined in the document
    let sections: Vec<&str> = sections_string.split("\n").collect();

    for section in sections {
        if section.is_empty() {
            continue
        }
        let mut find_settings = SectionData {
            begin: true,
            label: section,
        };

        let mut start = vec![];
        let mut end = vec![];
        if let Ok(()) = find_section(root, &mut start, &find_settings, &mut vec![]) {
            continue
        }

        find_settings.begin = false;
        if let Ok(()) = find_section(root, &mut end, &find_settings, &mut vec![]) {
            continue
        }

        if !start.is_empty() && !end.is_empty() {
            let inter = get_intermediary(&start, &end);
            let section = Section {
                title: String::from(section),
                position: Span {
                    start: inter.first().unwrap_or(root).get_position().start.clone(),
                    end: inter.last().unwrap_or(root).get_position().end.clone(),
                },
                tree: inter
            };
            writeln!(out, "{}\n", serde_yaml::to_string(&section)
                .expect("could not serialize section!"))?;
        }
    }
    Ok(())
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

/// Parameters for session finding.
struct SectionData<'a> {
    pub label: &'a str,
    pub begin: bool,
}

/// Return a path to the start / end of a section
fn find_section<'a>(root: &'a Element,
                    path: &mut Vec<&'a Element>,
                    settings: &SectionData<'a>,
                    out: &mut io::Write) -> io::Result<()> {

    path.push(root);
    if let &Element::HtmlTag { ref name, ref attributes, .. } = root {
        if name.to_lowercase() == "section" {
            for attr in attributes {
                if attr.key.to_lowercase() == if settings.begin {"begin"} else {"end"}
                    && attr.value.to_lowercase() == settings.label.to_lowercase() {
                    // abort recursion, preserve path
                    return Err(io::Error::new(io::ErrorKind::Other, "recusion abort"));
                }
            }
        }
    };
    // pop if recursion not aborted
    traverse_with(&find_section, root, path, settings, out)?;
    path.pop();
    Ok(())
}

/// Extract all child nodes from an elment in a list.
/// If an element has multiple fields, they are concatenated
/// in a semantically useful order.
fn extract_content<'a>(root: Element) -> Option<Vec<Element>> {
    match root {
        Element::Document { content, .. } => Some(content),
        Element::Heading { mut caption, mut content, .. } => {
            caption.append(&mut content);
            Some(caption)
        },
        Element::Formatted { content, .. } => Some(content),
        Element::Paragraph { content, .. } => Some(content),
        Element::Template { mut name, mut content, .. } => {
            name.append(&mut content);
            Some(name)
        },
        Element::TemplateArgument { value, .. } => Some(value),
        Element::InternalReference { mut target, options, mut caption, .. } => {
            for mut option in options {
                target.append(&mut option);
            }
            target.append(&mut caption);
            Some(target)
        },
        Element::ExternalReference { caption, .. } => Some(caption),
        Element::ListItem { content, .. } => Some(content),
        Element::List { content, .. } => Some(content),
        Element::Table { mut caption, mut rows, .. } => {
            caption.append(&mut rows);
            Some(caption)
        }
        Element::TableRow { cells, .. } => Some(cells),
        Element::TableCell { content, .. } => Some(content),
        Element::HtmlTag { content, .. } => Some(content),
        Element::Text { .. } => None,
        Element::Comment { .. } => None,
        Element::Error { .. } => None,
    }
}
