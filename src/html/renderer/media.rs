use super::HtmlRenderer;
use crate::preamble::*;

impl<'e, 's: 'e, 't: 'e, 'a> HtmlRenderer<'e, 't, 's, 'a> {
    pub fn internal_ref(
        &mut self,
        root: &'e InternalReference,
        out: &mut dyn io::Write,
    ) -> io::Result<bool> {
        // embedded (media) files
        if is_file(root, self.settings) {
            let meta = load_media_meta(&root.target, self.settings);
            let authors = meta.license.authors.join(", ");

            if is_centered(root) {
                let image_path =
                    mapped_media_path(self.html.target_type(), &root.target, self.settings);
                let caption_content = root.caption.render(self)?;
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

    pub fn gallery(&mut self, _root: &'e Gallery, out: &mut dyn io::Write) -> io::Result<bool> {
        self.write_error("galleries not implemented, yet!", out)?;
        Ok(false)
    }
}
