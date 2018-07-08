use super::HtmlRenderer;
use mediawiki_parser::*;
use preamble::*;



impl<'e, 's: 'e, 't: 'e> HtmlRenderer<'e, 't> {
    pub fn list(&mut self,
    root: &'e List,
    settings: &'s Settings,
    out: &mut io::Write,
) -> io::Result<bool> {

    let kind = if let Some(&Element::ListItem(ref li)) = root.content.first() {
        li.kind
    } else {
        self.write_error(
            "first child of list element \
             is not a list item (or does not exist)!",
            out,
        )?;
        return Ok(false);
    };
    match kind {
        ListItemKind::Ordered => {
                                    writeln!(out, "<ol class=\"ordered\">")?;
                                    for child in &root.content {
                                        if let Element::ListItem(ref li) = *child {
                                            write!(out, "<li>")?;
                                            self.run_vec(&li.content, settings, out)?;
                                            write!(out, "</li>")?;
                                        }
                                    }
                                    writeln!(out, "</ol>")?;
                                },
        ListItemKind::Unordered => {
                                    writeln!(out, "<ul class=\"unordered\">")?;
                                    for child in &root.content {
                                        if let Element::ListItem(ref li) = *child {
                                            write!(out, "<li>")?;
                                            self.run_vec(&li.content, settings, out)?;
                                            write!(out, "</li>")?;
                                        }
                                    }
                                    writeln!(out, "</ul>")?;
                                },
        ListItemKind::Definition | ListItemKind::DefinitionTerm => {
                                                                        writeln!(out, "<dl class=\"definitionlist\">")?;
                                                                        for child in &root.content {
                                                                            if let Element::ListItem(ref li) = *child {
                                                                                match li.kind {
                                                                                    ListItemKind::Definition => {
                                                                                        write!(out, "<dt class=\"definition\">")?;
                                                                                        self.run_vec(&li.content, settings, out)?;
                                                                                        write!(out, "</dt>")?;
                                                                                    },
                                                                                    ListItemKind::DefinitionTerm => {
                                                                                        write!(out, "<dd class=\"definitionterm\">")?;
                                                                                        self.run_vec(&li.content, settings, out)?;
                                                                                        write!(out, "</dd>")?;
                                                                                    },
                                                                                    _ => {
                                                                                        let msg = format!("error: different type of listElement in definitionList {:?}", &li.kind);
                                                                                        self.write_error(&msg, out)?;
                                                                                }
                                                                                }
                                                                            }
                                                                        }

                                                                    }
    };


    Ok(false)

}
}
