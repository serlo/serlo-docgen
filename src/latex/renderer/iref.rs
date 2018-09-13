//! Render internal references (embedded files, links, ...)

use super::LatexRenderer;
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
        let target_path = path::Path::new(&target_str);

        let doctitle = &settings.runtime.document_title;

        // embedded files (images, videos, ...)
        if is_file(root, settings) {
            let image_path =
                mapped_media_path(self.latex, &root.target, settings, PathMode::RELATIVE);
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
                    &license_text,
                    &image_path.to_string_lossy()
                )?;
            }

            return Ok(false);
        }

        // export links to other articles as url to the article
        if target_str
            .trim()
            .trim_left_matches(":")
            .to_lowercase()
            .starts_with("mathe für nicht-freaks:")
        {
            let cap_content = root.caption.render(self, settings)?;

            let mut url = settings.general.article_url_base.to_owned();
            url.push_str(&target_str);
            url = escape_latex(&urlencode(&url));

            writeln!(out, INTERNAL_HREF!(), &url, &cap_content)?;
            return Ok(false);
        }

        // anchor references
        let anchor_prefixes = ["#Anchor:", "#anchor:", "#Anker:", "#anker:"];
        let anchor_prefix = anchor_prefixes
            .iter()
            .filter(|p| target_str.starts_with(*p))
            .next();
        if let Some(prefix) = anchor_prefix {
            let target = target_str.trim_left_matches(prefix);
            write!(out, LABEL_REF!(), target.trim())?;
            return Ok(false);
        }

        let msg = format!("No export function defined for ref {:?}", target_path);
        self.write_error(&msg, out)?;
        Ok(false)
    }
}
