//! Render internal references (embedded files, links, ...)

use super::LatexRenderer;
use crate::preamble::*;
use base64;
use std::path;

impl<'e, 's: 'e, 't: 'e, 'a> LatexRenderer<'e, 't, 's, 'a> {
    pub fn get_license_text(&mut self, root: &'e InternalReference) -> io::Result<Option<String>> {
        let meta = load_media_meta(&root.target, &self.settings);
        let authors = meta.license.authors.join(", ");
        let license_text = format!(
            LICENSE_TEXT!(),
            &Self::escape_latex(&meta.license.url),
            &Self::escape_latex(
                &path::PathBuf::from(&meta.license.url)
                    .file_name()
                    .map(|f| f.to_string_lossy())
                    .unwrap_or_default(),
            ),
            &Self::escape_latex(&authors),
            &Self::escape_latex(&meta.license.shortname),
        );
        Ok(Some(license_text))
    }

    pub fn internal_ref(
        &mut self,
        root: &'e InternalReference,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        let target_str = extract_plain_text(&root.target);

        let doctitle = &self.args.document_title;

        // embedded files (images, videos, ...)
        if is_file(root, self.settings) {
            let image_path =
                mapped_media_path(self.latex.target_type(), &root.target, self.settings);
            let license_text = match self.get_license_text(root)? {
                Some(s) => s,
                None => return Ok(false),
            };

            // collect image options
            let mut image_options = vec![];
            for option in &root.options {
                image_options.push(extract_plain_text(option).trim().to_string());
            }

            if is_thumb(root) {
                let msg = "Thumbnail images should have been moved into galleries.";
                self.write_error(msg, out)?;
                eprintln!("error!");
                return Ok(false);
            }

            let cap_content = &root.caption.render(self)?;

            if is_centered(root) {
                self.write_def_location(&root.position, doctitle, out)?;

                let mut fig_content = format!(
                    FIGURE_CONTENT!(),
                    &image_options,
                    &license_text,
                    self.latex.image_width,
                    self.latex.image_height,
                    &image_path.to_string_lossy(),
                );

                if self.latex.centered_image_captions || iref_has_option(root, &["from_thumb"]) {
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
                    &license_text,
                    &image_path.to_string_lossy()
                )?;
            }

            return Ok(false);
        }

        let cap_content = root.caption.render(self)?;
        self.internal_link(&target_str, &cap_content, out)
    }

    /// only internal links, no embedded files. Does not require the root
    /// lifetime, thus can be called on temporary elements.
    pub fn internal_link(
        &mut self,
        target: &str,
        caption: &str,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        let target = target.trim().trim_left_matches(':').to_string();

        // internal references contained in the book.
        let anchor = matching_anchor(&target, &self.args.available_anchors);
        if let Some(anchor) = anchor {
            write!(out, LABEL_REF!(), &base64::encode(&anchor), &caption)?;
            return Ok(false);
        }

        // other internal references to mediawiki
        let mut url = self.settings.article_url_base.clone();
        url.push_str(&target);
        url = Self::escape_latex(&urlencode(&url));

        writeln!(out, INTERNAL_HREF!(), &url, &caption)?;
        Ok(false)
    }
}
