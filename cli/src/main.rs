use engine::{DatabaseConfig, QueryResult, SqlClient, Value};
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    println!("Minimalist SQL Runner");

    // Kết nối SQLite in-memory database
    let config = DatabaseConfig::sqlite("sqlite::memory:");

    let client = match SqlClient::connect(config).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Không thể kết nối: {e}");
            return;
        }
    };

    println!("Đã kết nối tới {}\n", client.database_type());
    println!("Gõ SQL query. Gõ 'exit' hoặc 'quit' để thoát.\n");

    loop {
        print!("sql> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Lỗi đọc input");
            continue;
        }

        let query = input.trim();

        if query.eq_ignore_ascii_case("exit") || query.eq_ignore_ascii_case("quit") {
            println!("Tạm biệt!");
            break;
        }

        if query.is_empty() {
            continue;
        }

        match client.execute(query).await {
            Ok(QueryResult::Query { columns, rows }) => {
                if columns.is_empty() {
                    println!("(empty result set)\n");
                } else {
                    print_table(&columns, &rows);
                }
            }
            Ok(QueryResult::Execution {
                rows_affected,
                last_insert_rowid,
            }) => {
                println!("{} rows affected", rows_affected);
                if let Some(id) = last_insert_rowid {
                    println!("last_insert_rowid: {id}");
                }
                println!();
            }
            Err(e) => {
                eprintln!("{e}\n");
            }
        }
    }
}

/// In kết quả query dạng bảng.
fn print_table(columns: &[engine::Column], rows: &[engine::Row]) {
    // Tính độ rộng tối đa cho mỗi cột
    let mut widths: Vec<usize> = columns.iter().map(|c| c.name.len()).collect();

    for row in rows {
        for (i, val) in row.values.iter().enumerate() {
            let len = match val {
                Some(v) => format_value(v).len(),
                None => 4, // "NULL".len()
            };
            if i < widths.len() {
                widths[i] = widths[i].max(len);
            }
        }
    }

    // Header
    print!("|");
    for (i, col) in columns.iter().enumerate() {
        print!(" {:<width$} |", col.name, width = widths[i]);
    }
    println!();

    // Separator
    print!("|");
    for w in &widths {
        print!("{}|", "-".repeat(w + 2));
    }
    println!();

    // Rows
    for row in rows {
        print!("|");
        for (i, val) in row.values.iter().enumerate() {
            let s = match val {
                Some(v) => format_value(v),
                None => "NULL".to_string(),
            };
            print!(" {:<width$} |", s, width = widths[i]);
        }
        println!();
    }

    println!("({} rows)\n", rows.len());
}

/// Format Value thành string hiển thị.
fn format_value(val: &Value) -> String {
    match val {
        Value::Integer(v) => v.to_string(),
        Value::Float(v) => v.to_string(),
        Value::Text(v) => v.clone(),
        Value::Blob(v) => format!("<{} bytes>", v.len()),
    }
}
