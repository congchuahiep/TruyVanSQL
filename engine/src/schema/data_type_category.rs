/// Phân loại kiểu dữ liệu SQL theo hành vi trong UI.
///
/// Mỗi driver implement `categorize()` để map kiểu riêng → category chung.
/// Category quyết định:
/// - Empty input → NULL hay empty string?
/// - Validate input như thế nào?
/// - Format SQL value ra sao?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataTypeCategory {
    /// Chuỗi ký tự: VARCHAR, TEXT, CHAR, NVARCHAR, CLOB...
    /// Empty input → Some("") (chuỗi rỗng hợp lệ)
    Text,
    /// Số nguyên: INTEGER, BIGINT, SMALLINT, SERIAL...
    /// Empty input → None (NULL)
    Integer,
    /// Số thực: REAL, FLOAT, DOUBLE, DECIMAL, NUMERIC...
    /// Empty input → None (NULL)
    Float,
    /// Boolean: BOOLEAN, BOOL...
    /// Empty input → None (NULL)
    Boolean,
    /// Ngày/giờ: DATE, TIME, TIMESTAMP, INTERVAL...
    /// Empty input → None (NULL)
    DateTime,
    /// Dữ liệu nhị phân: BLOB, BYTEA...
    /// Empty input → None (NULL)
    Binary,
    /// UUID
    /// Empty input → None (NULL)
    Uuid,
    /// Không xác định (computed column, kiểu lạ)
    /// Empty input → None (NULL) — an toàn hơn
    Unknown,
}

impl DataTypeCategory {
    /// Kiểm tra giá trị text có hợp lệ cho category này không.
    pub fn validate(&self, text: &str) -> bool {
        match self {
            DataTypeCategory::Integer => text.parse::<i64>().is_ok(),
            DataTypeCategory::Float => text.parse::<f64>().is_ok(),
            DataTypeCategory::Boolean => {
                text.eq_ignore_ascii_case("true")
                    || text.eq_ignore_ascii_case("false")
                    || text == "0"
                    || text == "1"
            }
            DataTypeCategory::Text | DataTypeCategory::Unknown => true,
            DataTypeCategory::DateTime => false, // TODO: Cần triển khai date parsing
            DataTypeCategory::Binary => false,   // TODO: Cần triển khai hex validation
            DataTypeCategory::Uuid => false,     // TODO: Cần triển khai UUID parsing
        }
    }

    /// Empty input có nên trở thành chuỗi rỗng (Some("")) thay vì NULL (None)?
    pub fn allows_empty_string(&self) -> bool {
        matches!(self, DataTypeCategory::Text)
    }
}
