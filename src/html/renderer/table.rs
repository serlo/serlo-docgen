use super::HtmlRenderer;
use crate::preamble::*;
use mediawiki_parser::*;

impl<'e, 's: 'e, 't: 'e, 'a> HtmlRenderer<'e, 't, 's, 'a> {
    pub fn table_cell(&mut self, root: &'e TableCell, out: &mut io::Write) -> io::Result<bool> {
        if root.header {
            write!(out, "<th")?;
            for attribute in &root.attributes {
                write!(
                    out,
                    " {}=\"{}\"",
                    &Self::escape_html(&attribute.key),
                    &Self::escape_html(&attribute.value)
                )?;
            }
            writeln!(out, ">")?;
            self.run_vec(&root.content, (), out)?;
            write!(out, "</th>")?;
        } else {
            write!(out, "<td")?;
            for attribute in &root.attributes {
                write!(
                    out,
                    " {}=\"{}\"",
                    &Self::escape_html(&attribute.key),
                    &Self::escape_html(&attribute.value)
                )?;
            }
            writeln!(out, ">")?;
            self.run_vec(&root.content, (), out)?;
            write!(out, "</td>")?;
        }
        Ok(false)
    }

    pub fn table_row(&mut self, root: &'e TableRow, out: &mut io::Write) -> io::Result<bool> {
        writeln!(out, "<tr")?;
        for attribute in &root.attributes {
            write!(
                out,
                " {}=\"{}\"",
                &Self::escape_html(&attribute.key),
                &Self::escape_html(&attribute.value)
            )?;
        }
        writeln!(out, ">")?;
        for element in &root.cells {
            match element {
                Element::TableCell(_) => {
                    self.run(element, (), out)?;
                }
                _ => {
                    let msg = format!(
                        "error: different type of element in root.cells in tablerow {:?}",
                        &root.cells
                    );
                    self.write_error(&msg, out)?;
                }
            }
        }
        writeln!(out, "</tr>")?;
        Ok(false)
    }
    pub fn table(&mut self, root: &'e Table, out: &mut io::Write) -> io::Result<bool> {
        write!(out, "<table")?;
        for attribute in &root.attributes {
            write!(
                out,
                " {}=\"{}\"",
                &Self::escape_html(&attribute.key),
                &Self::escape_html(&attribute.value)
            )?;
        }
        write!(out, ">")?;
        for element in &root.rows {
            match element {
                Element::TableRow(_) => {
                    self.run(element, (), out)?;
                }
                _ => {
                    let msg = format!(
                        "error: different type of element in root.rows in table {:?}",
                        &root.rows
                    );
                    self.write_error(&msg, out)?;
                }
            }
        }

        writeln!(out, "</table>")?;

        Ok(false)
    }
}
