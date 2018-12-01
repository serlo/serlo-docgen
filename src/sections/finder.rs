//! Implements helpters for the sections target related to finding
//! things in the ast structure.

use preamble::*;

/// Return a path to the start / end of a section
#[derive(Default)]
pub struct SectionFinder<'e, 'a> {
    /// label of the section to find.
    pub label: &'a str,
    /// get start or end of section?
    pub begin: bool,
    path: Vec<&'e Element>,
    /// the resulting path.
    pub result: Option<Vec<&'e Element>>,
}

impl<'e, 'a> Traversion<'e, ()> for SectionFinder<'e, 'a> {
    path_methods!('e);

    fn work(&mut self, root: &'e Element, _: (), _: &mut io::Write) -> io::Result<bool> {
        // end recursion if result is found
        if !self.result.is_none() {
            return Ok(false);
        }

        if let Element::HtmlTag(ref tag) = *root {
            if tag.name.to_lowercase() == "section" {
                for attr in &tag.attributes {
                    let norm = |s: &str| s.trim().to_lowercase().replace(' ', "_");
                    if attr.key.to_lowercase() == if self.begin { "begin" } else { "end" }
                        && norm(&attr.value) == norm(self.label)
                    {
                        self.result = Some(self.path.clone());
                    }
                }
            }
        };
        Ok(true)
    }
}

impl<'a, 'e> SectionFinder<'e, 'a> {
    fn find_path(root: &'e Element, label: &'a str, begin: bool) -> Option<Vec<&'e Element>> {
        let mut finder = SectionFinder {
            label,
            begin,
            path: vec![],
            result: None,
        };

        if finder.run(root, (), &mut vec![]).is_ok() {
            finder.result
        } else {
            None
        }
    }
    pub fn get_start(root: &'e Element, label: &'a str) -> Option<Vec<&'e Element>> {
        SectionFinder::find_path(root, label, true)
    }
    pub fn get_end(root: &'e Element, label: &'a str) -> Option<Vec<&'e Element>> {
        SectionFinder::find_path(root, label, false)
    }
}
