use prettytable::{Table, Row, Cell, format};
use crate::utils::db::ensure_db_ready;
use crate::utils::logger::info;
use crate::db::programs::list_programs;
use prettytable::row;


pub async fn handle_list() -> Result<(), Box<dyn std::error::Error>> {
    let pool = ensure_db_ready().await?;
    let programs = list_programs(&pool).await?;

    if programs.is_empty() {
        info("No programs found in database.");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    // Header
    table.set_titles(row!["ID", "Title", "Version", "Status", "Path"]);

    for p in programs {
        table.add_row(Row::new(vec![
            Cell::new(&p.id.to_string()),
            Cell::new(&p.title),
            Cell::new(&p.version),
            Cell::new(&p.status),
            Cell::new(&p.path),
        ]));
    }

    table.printstd();
    Ok(())
}
