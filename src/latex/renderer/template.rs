//! Implements template rendering for latex.

use preamble::*;
use super::LatexRenderer;


impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn template(&mut self, root: &'e Element,
                       settings: &'s Settings,
                       out: &mut io::Write) -> io::Result<bool> {

        if let Element::Template {
            ref name,
            ref content,
            ref position
        } = *root {

            let template_name = extract_plain_text(name);

            let doctitle = &settings.document_title;
            let envs = &self.latex.environments;

            // export simple environment templates
            if let Some(envs) = envs.get(&template_name) {

                let title = if let Some(title) = find_arg(content, "title") {
                    title.render(self, settings)?
                } else {
                    "".into()
                };

                for environment in envs {
                    if let Some(content) = find_arg(content, environment) {

                        self.write_def_location(content.get_position(), doctitle, out)?;
                        let content = content.render(self, settings)?;

                        self.environment(
                            environment,
                            &vec![title.trim()],
                            content.trim(),
                            out
                        )?;
                    }
                }
                return Ok(false);
            }

            // any other template
            match &template_name[..] {
                "formula" => {
                    self.formula(content, out)?;
                },
                "anchor" => {
                    write!(out, " {} ", escape_latex("<no anchors yet!>"))?;
                }
                _ => {
                    let message = format!("MISSING TEMPLATE: {}", template_name);
                    self.write_def_location(position, doctitle, out)?;
                    self.write_error(&message, out)?;
                }
            };
        }
        Ok(false)
    }

    fn formula(&self, content: &[Element], out: &mut io::Write) -> io::Result<()> {

        let mut math_text = "ERROR: Template was not transformed properly!";
        if let Some(&Element::TemplateArgument {
            ref value,
                ..
            }) = content.first() {
            if let Some(&Element::Text {
                ref text,
                ..
                }) = value.first() {
                math_text = trim_enclosing(text.trim(),
                                        "\\begin{align}",
                                        "\\end{align}");
                math_text = trim_enclosing(math_text,
                                        "\\begin{align*}",
                                        "\\end{align*}").trim();
            };
        };
        let indent = self.latex.indentation_depth;
        let width= self.latex.max_line_width;

        writeln!(out, "\\begin{{align*}}")?;
        writeln!(out, "{}", indent_and_trim(math_text, indent, width))?;
        writeln!(out, "\\end{{align*}}")
    }

    pub fn template_arg(&mut self, root: &'e Element,
                    settings: &'s Settings,
                    out: &mut io::Write) -> io::Result<bool> {

        if let Element::TemplateArgument {
            ref value,
            ..
        } = *root {
            self.run_vec(value, settings, out)?;
        }
        Ok(false)
    }

}
