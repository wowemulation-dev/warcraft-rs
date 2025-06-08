//! Table formatting utilities

use prettytable::{Cell, Row, Table};

/// Create a table with headers
#[allow(dead_code)]
pub fn create_table(headers: Vec<&str>) -> Table {
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    let header_cells: Vec<Cell> = headers
        .into_iter()
        .map(|h| Cell::new(h).style_spec("b"))
        .collect();
    table.set_titles(Row::new(header_cells));

    table
}

/// Add a row to a table
#[allow(dead_code)]
pub fn add_table_row(table: &mut Table, cells: Vec<String>) {
    let row_cells: Vec<Cell> = cells.into_iter().map(|s| Cell::new(&s)).collect();
    table.add_row(Row::new(row_cells));
}
