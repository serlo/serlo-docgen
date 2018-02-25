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

            let items = {
                let mut items = vec![];
                let mut definition_term = None;

                for child in content {
                    if let Element::ListItem { ref content, ref kind, .. } = *child {

                        let child_content = content.render(self, settings)?;

                        // definition term
                        if let ListItemKind::DefinitionTerm = *kind {
                            definition_term = Some(child_content);
                            continue
                        }

                        let item = if let ListItemKind::Definition = *kind {
                            format!(ITEM_DEFINITION!(),
                                definition_term.unwrap_or_default(),
                                child_content.trim()
                            )
                        } else {
                            format!(ITEM!(), child_content.trim())
                        };

                        definition_term = None;
                        items.push(item);
                    };
                }
                items
            };

            format!(out, LIST!(),
                envname,
                &items.join("\n"),
                envname
            )?;
        };
        Ok(false)
    }
}
