//! Render image galleries

use preamble::*;
use super::LatexRenderer;

#[derive(Debug)]
struct TableInfo<'e> {
    width: usize,
    header: Option<&'e [Element]>,
    body: &'e [Element],
}

impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {

    pub fn table_cell(
        &mut self,
        root: &'e Element,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {

        if let Element::TableCell {
            ref content,
            ..
        } = *root {
            // paragraphs in tables do not translate well for latex
            for child in content {
                if let Element::Paragraph { ref content, .. } = *child {
                    self.run_vec(content, settings, out)?;
                } else {
                    self.run(child, settings, out)?;
                }
            }
        }
        Ok(false)
    }

    pub fn table_row(
        &mut self,
        root: &'e Element,
        settings: &'s Settings,
        out: &mut io::Write
    ) -> io::Result<bool> {

        if let Element::TableRow {
            ref cells,
            ..
        } = *root {

            for (index, cell) in cells.iter().enumerate() {
                if index > 0 {
                    write!(out, " & ")?;
                }
                self.run(cell, settings, out)?;
            }
            writeln!(out, "\\\\")?;
        }
        Ok(false)
    }

    fn get_table_params(
        &mut self,
        rows: &'e [Element],
        out: &mut io::Write
    ) -> io::Result<Option<TableInfo<'e>>> {

        let mut table_width = None;
        let mut last_header_position = None;

        for (index, row) in rows.iter().enumerate() {
            let current_width = if let Element::TableRow { ref cells, .. } = *row {

                let is_header_row = cells.iter().fold(true, | con, e |
                    con && if let Element::TableCell { header, .. } = *e { header } else {false}
                );

                if is_header_row {
                    last_header_position = Some(index + 1);
                };
                cells.len()
            } else {
                self.write_error("row element is not TableRows!", out)?;
                return Ok(None)
            };

            if table_width.is_none() {
                table_width = Some(current_width);
            }

            if let Some(width) = table_width {
                if width != current_width {
                    self.write_error("inconsistent table row cell count!", out)?;
                    return Ok(None)
                }
            }
        }

        // only the first row can be header for now
        eprintln!("last header index: {:?}", last_header_position);

        let rows_split = rows.split_at(last_header_position.unwrap_or(0));
        Ok(Some(TableInfo {
            width: table_width.unwrap_or(0),
            header: if rows_split.0.len() > 0 { Some(rows_split.0) } else { None },
            body: rows_split.1
        }))
    }

    pub fn table(
        &mut self,
        root: &'e Element,
        settings: &'s Settings,
        out: &mut io::Write) -> io::Result<bool> {

        if let Element::Table {
            ref position,
            ref caption,
            ref rows,
            ..
        } = *root {

            self.write_def_location(position, &settings.document_title, out)?;
            let table_info = if let Some(info) = self.get_table_params(rows, out)? {
                info
            } else {
                return Ok(false)
            };

            let columns = "X[l]".repeat(table_info.width);

            let content = if let Some(header) = table_info.header {
                format!(TABLE_WITH_HEADER!(),
                    header.render(self, settings)?,
                    table_info.body.render(self, settings)?,
                )
            } else {
                format!(TABLE_WITHOUT_HEADER!(),
                    table_info.body.render(self, settings)?,
                )
            };

            let line_width = self.latex.max_line_width;
            let indent = self.latex.indentation_depth;

            writeln!(
                out,
                TABLE!(),
                columns,
                caption.render(self, settings)?.trim(),
                &indent_and_trim(&content, indent, line_width),
            )?;
        }
        Ok(false)
    }
}
