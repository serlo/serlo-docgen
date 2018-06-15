//! Render internal references (embedded files, links, ...)

use preamble::*;
use std::path;
use super::LatexRenderer;


impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn internal_ref(
        &mut self,
        root: &'e InternalReference,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {

        let target_str = extract_plain_text(&root.target);
        let target_path = path::Path::new(&target_str);

        let doctitle = &settings.runtime.document_title;

        // embedded files (images, videos, ...)
        if is_file(root, settings) {

            let image_path = build_image_path(self.latex, &root.target, settings);

            // collect image options
            let mut image_options = vec![];
            for option in &root.options {
                image_options.push(extract_plain_text(option).trim().to_string());
            }

            if is_thumb(root) {
                let msg = "Thumbnail images should have been moved into galleries.";
                self.write_error(msg, out)?;
                return Ok(false)
            }

            let cap_content = &root.caption.render(self, settings)?;

            if is_centered(root) {

                self.write_def_location(&root.position, doctitle, out)?;

                let mut fig_content = format!(
                    FIGURE_CONTENT!(),
                    &image_options,
                    self.latex.image_width,
                    self.latex.image_height,
                    &image_path,
                );

                if self.latex.centered_image_captions {
                    fig_content.push('\n');
                    fig_content.push_str(&format!(FIGURE_CAPTION!(), &cap_content));
                }

                self.environment("figure", &["H"], &fig_content, out)?;
            // inline images
            } else {
                writeln!(
                    out,
                    FIGURE_INLINE!(),
                    &image_options,
                    &image_path,
                )?;
            }

            return Ok(false)
        }

        // export links to other articles as url to the article
        if target_str.to_lowercase().starts_with("mathe für nicht-freaks:") {

            let cap_content = root.caption.render(self, settings)?;

            let mut url = settings.general.article_url_base.to_owned();
            url.push_str(&target_str);
            url = url.replace(' ', "_");

            writeln!(out, INTERNAL_HREF!(), &url, &cap_content)?;
            return Ok(false)
        }

        let msg = format!("No export function defined for ref {:?}", target_path);
        self.write_error(&msg, out)?;
        Ok(false)
    }
}
