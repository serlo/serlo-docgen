//! Helpers for the sections target related to extracting section content.

use mediawiki_parser::transformations::*;
use preamble::*;
use sections::finder::SectionFinder;

/// Paramters for section filtering transformation.
#[derive(Debug, Clone)]
pub struct SectionFilter<'b, 'e: 'b> {
    pub begin: &'b Vec<&'e Element>,
    pub end: &'b Vec<&'e Element>,
    /// wether to include nodes before end of the section,
    /// even if no start is present in the current subtree.
    pub include_pre: bool,
}

impl<'a, 'b: 'a> SectionFilter<'a, 'b> {
    /// Extract a list of nodes forming a section from an input ast.
    pub fn extract(label: &str, root: &Element) -> Option<Vec<Element>> {
        let start = SectionFinder::get_start(root, label);
        let end = SectionFinder::get_end(root, label);
        let (start, end) = match (start, end) {
            (Some(start), Some(end)) => (start, end),
            _ => return None,
        };

        // lowest common node
        let mut common = None;
        for ps in start.iter().rev() {
            for pe in end.iter().rev() {
                if pe == ps {
                    common = Some(ps);
                    break;
                }
            }
            if common.is_some() {
                break;
            }
        }
        let common = match common {
            Some(c) => c,
            None => return None,
        };

        let filter = SectionFilter {
            begin: &start,
            end: &end,
            include_pre: false,
        };

        let result =
            filter_section_element(common, &[], &filter).expect("section extraction failed!");

        extract_content(result)
    }
}

/// Recursively trim a subtree to only contain the elements
/// enclosed by the section paths in `SectionFilter`.
fn filter_section_element(root: &Element, path: &[&Element], settings: &SectionFilter) -> TResult {
    recurse_clone_template(
        &filter_section_element,
        root,
        path,
        settings,
        &filter_section_subtree,
    )
}

/// Recursively trim a list of elments to only contain the elements
/// enclosed by the section paths in `SectionFilter`.
fn filter_section_subtree<'a>(
    _func: &TFunc<&SectionFilter>,
    content: &[Element],
    path: &[&'a Element],
    settings: &SectionFilter,
) -> TListResult {
    let mut result = vec![];
    let mut found_begin = false;

    for child in content {
        if settings.begin.contains(&child) {
            found_begin = true;

            // ignore the starting section tag
            if Some(&child) != settings.begin.last() {
                result.push(
                    filter_section_element(child, path, &settings.clone())
                        .expect("error in section filter"),
                );
            }
            continue;
        }
        if settings.end.contains(&child) {
            let mut child_settings = settings.clone();
            child_settings.include_pre = true;

            // ignore the ending section tag
            if Some(&child) != settings.end.last() {
                result.push(
                    filter_section_element(child, path, &child_settings)
                        .expect("error in section filter"),
                );
            }
            break;
        }

        if found_begin || settings.include_pre {
            result.push(child.clone());
        }
    }
    Ok(result)
}
