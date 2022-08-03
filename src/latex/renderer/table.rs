//! Render image galleries

use super::LatexRenderer;
use crate::preamble::*;

#[derive(Debug)]
struct TableInfo<'e> {
    width: usize,
    header: Option<&'e [Element]>,
    body: &'e [Element],
}

impl<'e, 's: 'e, 't: 'e, 'a> LatexRenderer<'e, 't, 's, 'a> {
    pub fn table_cell(&mut self, root: &'e TableCell, out: &mut dyn io::Write) -> io::Result<bool> {
        // paragraphs in tables do not translate well for latex
        self.run_vec_nopar(&root.content, out)?;
        Ok(false)
    }

    pub fn table_row(&mut self, root: &'e TableRow, out: &mut dyn io::Write) -> io::Result<bool> {
        for (index, cell) in root.cells.iter().enumerate() {
            if index > 0 {
                write!(out, " & ")?;
            }
            self.run(cell, (), out)?;
        }
        writeln!(out, "\\\\")?;
        Ok(false)
    }

    fn get_table_params(
        &mut self,
        rows: &'e [Element],
        out: &mut dyn io::Write,
    ) -> io::Result<Option<TableInfo<'e>>> {
        let mut table_width = None;
        let mut last_header_position = None;

        for (index, row) in rows.iter().enumerate() {
            let current_width = if let Element::TableRow(ref row) = *row {
                let is_header_row = row.cells.iter().all(|e| {
                    if let Element::TableCell(ref c) = *e {
                        c.header
                    } else {
                        false
                    }
                });

                if is_header_row {
                    last_header_position = Some(index + 1);
                };
                row.cells.len()
            } else {
                self.write_error("row element is not TableRows!", out)?;
                return Ok(None);
            };

            if table_width.is_none() {
                table_width = Some(current_width);
            }

            if let Some(width) = table_width {
                if width != current_width {
                    self.write_error("inconsistent table row cell count!", out)?;
                    return Ok(None);
                }
            }
        }

        let rows_split = rows.split_at(last_header_position.unwrap_or(0));
        Ok(Some(TableInfo {
            width: table_width.unwrap_or(0),
            header: if !rows_split.0.is_empty() {
                Some(rows_split.0)
            } else {
                None
            },
            body: rows_split.1,
        }))
    }

    pub fn table(&mut self, root: &'e Table, out: &mut dyn io::Write) -> io::Result<bool> {
        self.write_def_location(&root.position, &self.args.document_title, out)?;
        let table_info = if let Some(info) = self.get_table_params(&root.rows, out)? {
            info
        } else {
            return Ok(false);
        };

        let columns = "X[l]".repeat(table_info.width);

        let content = if let Some(header) = table_info.header {
            format!(
                TABLE_WITH_HEADER!(),
                header.render(self)?,
                table_info.body.render(self)?,
            )
        } else {
            table_info.body.render(self)?
        };

        let line_width = self.latex.max_line_width;
        let indent = self.latex.indentation_depth;

        writeln!(
            out,
            TABLE!(),
            columns,
            root.caption.render(self)?.trim(),
            &indent_and_trim(content.trim(), indent, line_width),
        )?;

        Ok(false)
    }
}
