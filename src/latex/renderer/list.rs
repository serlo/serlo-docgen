//! Render mediawiki lists.

use preamble::*;
use super::LatexRenderer;
use mediawiki_parser::*;


impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn list(&mut self, root: &'e Element,
            settings: &'s Settings,
            out: &mut io::Write) -> io::Result<bool> {

        if let Element::List { ref content, .. } = *root {

            let kind = if let Element::ListItem {
                ref kind,
                ..
            } = *content.first().unwrap_or(root) {
                    kind
            } else {
                self.write_error("first child of list element \
                    is not a list item (or does not exist)!", out)?;
                eprintln!("error: {:?}", root);
                return Ok(false)
            };

            let envname = match *kind {
                ListItemKind::Ordered => "enumerate",
                ListItemKind::Unordered
                | ListItemKind::Definition
                | ListItemKind::DefinitionTerm
                => "itemize",
            };
            writeln!(out, "\\begin{{{}}}", envname)?;

            let mut def_term_temp = String::new();

            for child in content {
                if let Element::ListItem { ref content, ref kind, .. } = *child {

                    // render paragraph content
                    let mut par_content = vec![];
                    self.run_vec(content, settings, &mut par_content)?;
                    let par_string = String::from_utf8(par_content)
                        .unwrap().trim_right().to_string();

                    // definition term
                    if let ListItemKind::DefinitionTerm = *kind {
                        def_term_temp.push_str(&par_string);
                        continue
                    }

                    let item_string = if let ListItemKind::Definition = *kind {
                        format!("\\item \\textbf{{{}}}: {}", def_term_temp, par_string)
                    } else {
                        format!("\\item {}", par_string)
                    };
                    def_term_temp = String::new();


                    let indent = self.latex.indentation_depth;
                    let line_width = self.latex.max_line_width;

                    // trim and indent output string
                    let trimmed = indent_and_trim(&item_string, indent, line_width);

                    writeln!(out, "{}", &trimmed)?;
                }
            }
            writeln!(out, "\\end{{{}}}\n", envname)?;
        };
        Ok(false)
    }
}
