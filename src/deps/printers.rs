//! Helpers which look for certain things in the input ast and print
//! them to a given output in `make` dependency format.

use preamble::*;

/// Prints paths of the sections included in a document.
#[derive(Default)]
pub struct InclusionPrinter<'b> {
    pub path: Vec<&'b Element>,
}

impl<'a, 'b: 'a> Traversion<'a, &'b Settings> for InclusionPrinter<'a> {
    path_methods!('a);

    fn work(
        &mut self,
        root: &Element,
        settings: &'b Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        if let Element::Template(ref template) = *root {
            let prefix: &str = &settings.general.section_inclusion_prefix;
            let template_name = extract_plain_text(&template.name);

            // section transclusion
            if template_name.to_lowercase().starts_with(&prefix) {
                let article = trim_prefix(&template_name, prefix)
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                let section_name = extract_plain_text(&template.content)
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                let path = get_section_path(&article, &section_name, settings);
                write!(out, "\\\n\t{}", &path)?;
            }
        };
        Ok(true)
    }
}

/// Print paths of file dependencies of an article.
pub struct FilesPrinter<'e, 't> {
    pub path: Vec<&'e Element>,
    /// map of original to target file extension of a dependency.
    pub target: &'t Target,
}

impl<'e, 's: 'e, 't> Traversion<'e, &'s Settings> for FilesPrinter<'e, 't> {
    path_methods!('e);

    fn work(
        &mut self,
        root: &Element,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        if let Element::InternalReference(ref iref) = *root {
            if !is_file(iref, settings) {
                return Ok(true);
            }

            let file_path = build_media_path(&iref.target, settings);
            let image_path =
                mapped_media_path(self.target, &iref.target, settings);
            write!(out, "\\\n\t{}", &image_path.to_string_lossy())?;
            write!(out, "\\\n\t{}.meta", &file_path.to_string_lossy())?;
        };
        Ok(true)
    }
}

impl<'e, 't> FilesPrinter<'e, 't> {
    pub fn new(target: &'t Target) -> FilesPrinter {
        FilesPrinter {
            path: vec![],
            target,
        }
    }
}
