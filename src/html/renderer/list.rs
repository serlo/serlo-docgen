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
                                                                        writeln!(out, "<dl class=\"definition\">")?;
                                                                        writeln!(out, "<dl class=\"definition\">")?;

                                                                    }
    };//here go with for loop trhough list, get content here, add list definition before, three different class styles


    Ok(false)

}
}
