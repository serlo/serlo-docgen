use super::HtmlRenderer;
use mediawiki_parser::*;
use preamble::*;

impl<'e, 's: 'e, 't: 'e> HtmlRenderer<'e, 't> {

    pub fn table_cell(
        &mut self,
        root: &'e TableCell,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        if root.header {
            write!(out, "<th>")?;
            self.run_vec(&root.content, settings, out)?;
            write!(out, "</th>")?;
        }
        else{
            write!(out, "<td>")?;
            self.run_vec(&root.content, settings, out)?;
            write!(out, "</td>")?;
        }
        Ok(false)
    }

    pub fn table_row(
        &mut self,
        root: &'e TableRow,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        writeln!(out, "<tr>");
        for element in &root.cells {
                match element {
                    TableCell => {
                                    self.run(element, settings, out)?;
                                },
                    _ => {
                            let msg = format!(
                                        "error: different type of element in root.cells in tablerow {:?}",&root.cells);
                            self.write_error(&msg, out)?;
                        }
                }
            }
            writeln!(out, "</tr>");
        Ok(false)
    }
    pub fn table(
        &mut self,
        root: &'e Table,
        settings: &'s Settings,
        out: &mut io::Write,
    ) -> io::Result<bool> {
        writeln!(out, "<table>")?;
        for element in &root.rows {
                match element {
                    TableRow => {
                                    self.run(element, settings, out)?;
                                },
                    _ => {
                            let msg = format!(
                                        "error: different type of element in root.rows in table {:?}",&root.rows);
                            self.write_error(&msg, out)?;
                        }
                }
            }

        writeln!(out, "</table>")?;


        Ok(false)
    }
}

