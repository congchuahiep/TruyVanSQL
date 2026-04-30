pub mod value;

pub use value::Value;

/// Kết quả thống nhất của mọi SQL query.
///
/// Phân biệt hai loại kết quả chính:
/// - **Execution**: DML/DDL trả về metadata (rows affected, last insert id)
/// - **Query**: DQL trả về dữ liệu dạng bảng (columns + rows)
///
/// Việc tách thành enum giúp caller xử lý type-safe:
/// ```ignore
/// match client.execute("SELECT * FROM users").await? {
///     QueryResult::Query { columns, rows } => print_table(columns, rows),
///     QueryResult::Execution { rows_affected, .. } => println!("{rows_affected} rows affected"),
/// }
/// ```
#[derive(Debug, Clone)]
pub enum QueryResult {
    /// Kết quả của DML/DDL (INSERT, UPDATE, DELETE, CREATE, DROP, ...).
    Execution {
        /// Số dòng bị ảnh hưởng
        rows_affected: u64,
        /// ID của dòng vừa insert (chỉ có ý nghĩa với INSERT)
        last_insert_rowid: Option<i64>,
    },
    /// Kết quả của DQL (SELECT, PRAGMA, EXPLAIN, WITH ...).
    Query {
        /// Danh sách cột
        columns: Vec<Column>,
        /// Danh sách dòng dữ liệu
        rows: Vec<Row>,
    },
}

/// Metadata của một cột trong kết quả query.
#[derive(Debug, Clone)]
pub struct Column {
    /// Tên cột
    pub name: String,
    /// Kiểu dữ liệu declared trong schema (có thể None nếu query tạo ra computed column)
    pub declared_type: Option<String>,
}

/// Một dòng dữ liệu trong kết quả query.
///
/// Mỗi `Row` chứa danh sách `Option<Value>`, thứ tự tương ứng với thứ tự cột.
/// `None` đại diện cho giá trị NULL.
#[derive(Debug, Clone)]
pub struct Row {
    pub values: Vec<Option<Value>>,
}
