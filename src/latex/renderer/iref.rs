//! Render internal references (embedded files, links, ...)

use preamble::*;
use std::path;
use super::LatexRenderer;


impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn build_image_path(
        &self,
        target: &[Element],
        settings: &Settings
    ) -> String {

        let target_str = extract_plain_text(target);
        let target_path = path::Path::new(&target_str);
        let ext = target_path.extension().unwrap_or_default();
        let ext_str = ext.to_string_lossy().into();
        let target_extension = self.latex
            .get_extension_mapping()
            .get(&ext_str)
            .unwrap_or(&ext_str);

        let file_path = path::PathBuf::from(&settings.external_file_path)
            .join(target_path
                .with_extension(&target_extension)
                .file_stem()
                .expect("image path is empty!"))
            .to_string_lossy()
            .to_string();
        filename_to_make(&file_path)
    }

    pub fn internal_ref(
        &mut self,
        root: &'e InternalReference,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {

        let target_str = extract_plain_text(&root.target);
        let target_path = path::Path::new(&target_str);
        let ext = target_path.extension().unwrap_or_default();
        let ext_str = ext.to_string_lossy().into();

        let doctitle = &settings.document_title;
        let file_exts = &settings.external_file_extensions;

        // file is embedded as an image
        if file_exts.contains(&ext_str) {

            let image_path = self.build_image_path(&root.target, settings);

            // collect image options
            let mut image_options = vec![];
            for option in &root.options {
                image_options.push(extract_plain_text(option).trim().to_string());
            }

            // thumnail images
            if image_options.contains(&"thumb".to_owned()) {
                let msg = "Thumbnail images should have been moved into galleries.";
                self.write_error(msg, out)?;
                return Ok(false)
            }

            let cap_content = &root.caption.render(self, settings)?;

            // centered images
            if image_options.contains(&"center".to_owned()) {

                self.write_def_location(&root.position, doctitle, out)?;

                let fig_content = format!(
                    FIGURE_CONTENT!(),
                    &image_options,
                    self.latex.image_width,
                    self.latex.image_height,
                    &image_path,
                    &cap_content
                );

                self.environment("figure", &["h"], &fig_content, out)?;
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

            let mut url = settings.article_url_base.to_owned();
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
