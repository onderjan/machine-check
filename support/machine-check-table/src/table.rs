use prettytable::{Attr, Cell, Row, Table};

use crate::{RunResult, RunStats, TaskStats, TestConfig};
pub(super) fn print_table(
    table_name: Option<String>,
    test_results: &Vec<(TestConfig, Vec<(TaskStats, RunStats)>)>,
) {
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    // add heading

    let mut heading = vec![Cell::new(&table_name.unwrap_or(String::new())).with_style(Attr::Bold)];
    for test_result in test_results {
        heading.push(
            Cell::new(&test_result.0.name.clone().unwrap_or_else(|| {
                String::from(match test_result.0.tool {
                    crate::Tool::MachineCheck => "machine-check",
                    crate::Tool::Abc => "ABC",
                    crate::Tool::Avr => "AVR",
                })
            }))
            .with_style(Attr::Bold),
        )
    }
    table.set_titles(Row::new(heading));

    // add other rows

    add_row(
        &mut table,
        test_results,
        "correct true",
        |result| matches!(result, Some(RunResult::Correct(true))),
        false,
    );
    add_row(
        &mut table,
        test_results,
        "correct false",
        |result| matches!(result, Some(RunResult::Correct(false))),
        false,
    );
    add_row(
        &mut table,
        test_results,
        "wrong",
        |result| matches!(result, Some(RunResult::Wrong(_))),
        true,
    );
    add_row(
        &mut table,
        test_results,
        "timeout",
        |result| matches!(result, Some(RunResult::Timeout)),
        false,
    );
    add_row(
        &mut table,
        test_results,
        "out of memory",
        |result| matches!(result, Some(RunResult::OutOfMemory)),
        false,
    );
    add_row(
        &mut table,
        test_results,
        "other error",
        |result| matches!(result, Some(RunResult::OtherError)),
        true,
    );

    add_row(
        &mut table,
        test_results,
        "missing",
        |result| matches!(result, None),
        true,
    );

    table.printstd();
}

fn add_row(
    table: &mut Table,
    test_results: &Vec<(TestConfig, Vec<(TaskStats, RunStats)>)>,
    name: &str,
    matches: impl Fn(&Option<RunResult>) -> bool,
    should_be_zero: bool,
) {
    let mut total_count = 0;
    let mut cells = vec![Cell::new(name).with_style(Attr::Bold)];
    for test_result in test_results {
        let count = count_results(&test_result.1, &matches);
        total_count += count;

        let mut cell = Cell::new(&count.to_string());
        if count > 0 && should_be_zero {
            cell = cell
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(prettytable::color::RED));
        }

        cells.push(cell)
    }
    if total_count > 0 || !should_be_zero {
        table.add_row(Row::new(cells));
    }
}

fn count_results(
    tests: &[(TaskStats, RunStats)],
    matches: impl Fn(&Option<RunResult>) -> bool,
) -> usize {
    tests.iter().filter(|test| matches(&test.1.result)).count()
}
