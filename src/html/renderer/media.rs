use super::HtmlRenderer;
use mwparser_utils::*;
use preamble::*;
use std::path;

impl<'e, 's: 'e, 't: 'e> HtmlRenderer<'e, 't> {
    pub fn internal_ref(
        &mut self,
        root: &'e InternalReference,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        let target_str = extract_plain_text(&root.target);
        let target_path = path::Path::new(&target_str);

        // embedded (media) files
        if is_file(root, settings) {
            let meta = load_media_meta(&root.target, settings);
            let authors = meta.license.authors.join(", ");

            if is_centered(root) {
                let image_path = mapped_media_path(self.html.target_type(), &root.target, settings);
                let caption_content = root.caption.render(self, settings)?;
                let license_link = format!(
                    "<a class=\"serlo-fig-license-url\" href=\"{}\">{}: {}</a>",
                    &urlencode(&meta.license.detailsurl),
                    &authors,
                    &meta.license.shortname
                );

                writeln!(out, "<figure class=\"serlo-fig-center\">")?;
                writeln!(
                    out,
                    "<img src=\"{}\"/>",
                    &urlencode(&image_path.to_string_lossy())
                )?;
                writeln!(
                    out,
                    "<figcaption>{} ({})</figcaption>",
                    &caption_content, &license_link
                )?;
                writeln!(out, "</figure>")?;
            } else {
                self.write_error("non-centered image not implemented, yet!", out)?;
            }

            return Ok(false);
        } else {
            self.write_error("internal links not implemented, yet!", out)?;
        }

        Ok(false)
    }

    pub fn gallery(
        &mut self,
        root: &'e Gallery,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        self.write_error("galleries not implemented, yet!", out)?;
        Ok(false)
    }
}
