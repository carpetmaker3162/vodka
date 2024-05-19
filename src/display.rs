use cli_table::{print_stdout, Cell, CellStruct, Style, Table};
use crate::Entry;
use crate::store::{get_all_rows, search_entries};

pub fn display_all() -> bool {
    let entries = get_all_rows();
    display(entries)
}

pub fn display_search(name: String, login: String) -> bool {
    let entries = search_entries(name, login);
    display(entries)
}

pub fn display(entries: Vec<Entry>) -> bool {
    let mut table: Vec<Vec<CellStruct>> = Vec::new();

    for entry in entries {
        table.push(entry.as_table_row())
    }

    let table = table
        .table()
        .title(vec![
            "Name".cell().bold(true),
            "Login".cell().bold(true),
            "Password".cell().bold(true),
            "Comment".cell().bold(true),
        ])
        .bold(true);

    print_stdout(table).is_ok()
}