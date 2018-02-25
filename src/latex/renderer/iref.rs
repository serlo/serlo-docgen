//! Render internal references (embedded files, links, ...)

use preamble::*;
use std::path;
use super::LatexRenderer;


impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn internal_ref(&mut self, root: &'e Element,
                    settings: &'s Settings,
                    out: &mut io::Write) -> io::Result<bool> {

        if let Element::InternalReference {
            ref target,
            ref options,
            ref caption,
            ref position
         } = *root {

            let target_str = extract_plain_text(target);
            let target_path = path::Path::new(&target_str);
            let ext = target_path.extension().unwrap_or_default();
            let ext_str = ext.to_string_lossy().into();

            let doctitle = &settings.document_title;
            let img_exts = &settings.image_extensions;

            // file is an image
            if img_exts.contains(&ext_str) {

                let image_path = path::PathBuf::from(&settings.image_path)
                    .join(target_path.file_stem()
                    .expect("image path is empty!"))
                    .to_string_lossy()
                    .to_string();
                let image_path = filename_to_make(&image_path);

                // collect image options
                let mut image_options = vec![];
                for option in options {
                    image_options.push(extract_plain_text(option).trim().to_string());
                }

                self.write_def_location(position, doctitle, out)?;

                let cap_content = caption.render(self, settings)?;

                writeln!(
                    out,
                    FIGURE_ENV!(),
                    &image_options,
                    self.latex.image_width,
                    self.latex.image_height,
                    &image_path,
                    &cap_content
                )?;
                return Ok(false)
            }

            // export links to other articles as url to the article
            if target_str.to_lowercase().starts_with("mathe für nicht-freaks:") {

                let cap_content = caption.render(self, settings)?;

                let mut url = settings.article_url_base.to_owned();
                url.push_str(&target_str);
                url = url.replace(' ', "_");

                writeln!(out, INTERNAL_HREF!(), &url, &cap_content)?;
                return Ok(false)
            }

            let msg = format!("No export function defined for ref {:?}", target_path);
            self.write_error(&msg, out)?;
        }
        Ok(false)
    }
}
