//! Render mediawiki lists.

use preamble::*;
use super::LatexRenderer;
use mediawiki_parser::*;


impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn list(
        &mut self,
        root: &'e List,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {

        let kind = if let Some(&Element::ListItem(ref li))
            = root.content.first() {
                li.kind
        } else {
            self.write_error("first child of list element \
                is not a list item (or does not exist)!", out)?;
            return Ok(false)
        };

        let envname = match kind {
            ListItemKind::Ordered => "enumerate",
            ListItemKind::Unordered
            | ListItemKind::Definition
            | ListItemKind::DefinitionTerm
            => "itemize",
        };

        let items = {
            let mut items = vec![];
            let mut definition_term = None;

            for child in &root.content {
                if let Element::ListItem(ref li) = *child {

                    let child_content = li.content.render(self, settings)?;

                    // definition term
                    if let ListItemKind::DefinitionTerm = li.kind {
                        definition_term = Some(child_content);
                        continue
                    }

                    let item = if let ListItemKind::Definition = li.kind {
                        format!(ITEM_DEFINITION!(),
                            definition_term.unwrap_or_default(),
                            child_content.trim()
                        )
                    } else {
                        format!(ITEM!(), child_content.trim())
                    };

                    let line_width = self.latex.max_line_width;
                    let indent = self.latex.indentation_depth;

                    definition_term = None;
                    items.push(indent_and_trim(&item, indent, line_width));
                };
            }
            items
        };

        writeln!(out, LIST!(),
            envname,
            &items.join("\n"),
            envname
        )?;
        Ok(false)
    }
}
