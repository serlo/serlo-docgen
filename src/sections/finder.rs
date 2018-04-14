//! Implements helpters for the sections target related to finding
//! things in the ast structure.

use preamble::*;

/// Collect the names of all beginning sections in a document.
#[derive(Default)]
pub struct SectionNameCollector<'e> {
    path: Vec<&'e Element>,
    pub sections: Vec<String>,
}

impl<'e> Traversion<'e, ()> for SectionNameCollector<'e> {

    path_methods!('e);

    fn work(&mut self,
            root: &'e Element,
            _: (),
            _: &mut io::Write) -> io::Result<bool> {

        if let Element::HtmlTag(ref tag) = *root {
            if tag.name.to_lowercase() == "section"  {
                for attr in &tag.attributes {
                    if attr.key == "begin" {
                        self.sections.push(attr.value.trim().into());
                    }
                }
            }
        };
        Ok(true)
    }
}

impl<'e> SectionNameCollector<'e> {
    pub fn collect_from(root: &Element) -> Vec<String> {
        let mut collector = SectionNameCollector::default();
        if collector.run(root, (), &mut vec![]).is_ok() {
            collector.sections
        } else {
            vec![]
        }
    }
}

/// Return a path to the start / end of a section
#[derive(Default)]
pub struct SectionFinder<'e, 'a> {
    /// label of the section to find.
    pub label: &'a str,
    /// get start or end of section?
    pub begin: bool,
    path: Vec<&'e Element>,
    /// the resulting path.
    pub result: Vec<&'e Element>
}

impl<'e, 'a> Traversion<'e, ()> for SectionFinder<'e, 'a> {

    path_methods!('e);

    fn work(&mut self,
            root: &'e Element,
            _: (),
            _: &mut io::Write) -> io::Result<bool> {

        // end recursion if result is found
        if !self.result.is_empty() {
            return Ok(false)
        }

        if let Element::HtmlTag(ref tag) = *root {
            if tag.name.to_lowercase() == "section" {
                for attr in &tag.attributes {
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
    fn find_path(root: &'e Element, label: &'a str, begin: bool) -> Vec<&'e Element> {
        let mut finder = SectionFinder {
            label,
            begin,
            path: vec![],
            result: vec![],
        };

        if finder.run(root, (), &mut vec![]).is_ok() {
            finder.result
        } else {
            vec![]
        }
    }
    pub fn get_start(root: &'e Element, label: &'a str) -> Vec<&'e Element> {
        SectionFinder::find_path(root, label, true)
    }
    pub fn get_end(root: &'e Element, label: &'a str) -> Vec<&'e Element> {
        SectionFinder::find_path(root, label, false)
    }
}

