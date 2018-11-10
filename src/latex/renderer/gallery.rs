//! Render image galleries

use super::LatexRenderer;
use preamble::*;

impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {
    pub fn gallery(
        &mut self,
        root: &'e Gallery,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        let doctitle = &settings.runtime.document_title;

        let mut rendered_images = vec![];

        for image in &root.content {
            if let Element::InternalReference(ref iref) = *image {
                let path =
                    mapped_media_path(self.latex, &iref.target, settings);
                let caption = iref.caption.render(self, settings)?;

                // collect image options
                let mut image_options = vec![];
                for option in &iref.options {
                    image_options.push(extract_plain_text(option).trim().to_string());
                }

                let license_text = match self.get_license_text(iref, settings, out)? {
                    Some(s) => s,
                    None => return Ok(false),
                };

                let mut inner = format!(
                    GALLERY_CONTENT!(),
                    &image_options,
                    0.9 / (self.latex.gallery_images_per_row as f64),
                    &license_text,
                    self.latex.image_height,
                    &path.to_string_lossy(),
                    &caption
                );

                let indent = self.latex.indentation_depth;
                let line_width = self.latex.max_line_width;
                rendered_images.push(inner);
            }
        }

        // partition gallery rows
        let mut table_rows = vec![];
        for chunk in rendered_images.chunks(self.latex.gallery_images_per_row) {
            let mut row = chunk.join("\\hfill\n");
            let missing = self.latex.gallery_images_per_row - chunk.len();
            if missing > 0 {
                row.push_str("\\hfill\\\\\n");
            }
            table_rows.push(row);
        }

        writeln!(out, "")?;
        self.write_def_location(&root.position, doctitle, out)?;
        let sep = &self.latex.paragraph_separator;
        writeln!(out, "{}{}\n", table_rows.join("\\\\\n"), sep)?;
        Ok(false)
    }
}
