//! Render internal references (embedded files, links, ...)

use super::LatexRenderer;
use base64;
use preamble::*;
use std::path;

impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {
    pub fn get_license_text(
        &mut self,
        root: &'e InternalReference,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<Option<String>> {
        let meta = match load_media_meta(&root.target, settings) {
            MetaLoadResult::Meta(m) => m,
            e @ _ => {
                let error = escape_latex(&format!("{:#?}", e));
                self.write_error(&error, out)?;
                return Ok(None);
            }
        };
        let authors = meta.license.authors.join(", ");
        let license_text = format!(
            LICENSE_TEXT!(),
            &escape_latex(&meta.license.url),
            &escape_latex(
                &path::PathBuf::from(&meta.license.url)
                    .file_name()
                    .map(|f| f.to_string_lossy())
                    .unwrap_or_default(),
            ),
            &escape_latex(&authors),
            &escape_latex(&meta.license.shortname),
        );
        Ok(Some(license_text))
    }

    pub fn internal_ref(
        &mut self,
        root: &'e InternalReference,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        let target_str = extract_plain_text(&root.target);

        let doctitle = &settings.runtime.document_title;

        // embedded files (images, videos, ...)
        if is_file(root, settings) {
            let image_path =
                mapped_media_path(self.latex, &root.target, settings);
            let license_text = match self.get_license_text(root, settings, out)? {
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

            let cap_content = &root.caption.render(self, settings)?;

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

        let cap_content = root.caption.render(self, settings)?;
        self.internal_link(&target_str, &cap_content, settings, out)
    }

    /// only internal links, no embedded files. Does not require the root
    /// lifetime, thus can be called on temporary elements.
    pub fn internal_link(
        &mut self,
        target: &str,
        caption: &str,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        let target = target.trim().trim_left_matches(":").to_string();

        // internal references contained in the book.
        let anchor = matching_anchor(&target, &settings.runtime.available_anchors);
        if let Some(anchor) = anchor {
            write!(out, LABEL_REF!(), &base64::encode(&anchor), &caption)?;
            return Ok(false);
        }

        // other internal references to mediawiki
        let mut url = settings.general.article_url_base.clone();
        url.push_str(&target);
        url = escape_latex(&urlencode(&url));

        writeln!(out, INTERNAL_HREF!(), &url, &caption)?;
        return Ok(false);
    }
}
