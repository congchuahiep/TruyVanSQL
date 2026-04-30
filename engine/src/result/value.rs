/// Đại diện cho giá trị của một cell trong kết quả query.
///
/// Mỗi variant map tới một nhóm SQL type cơ bản.
/// Database driver chịu trách nhiệm convert native type -> `Value`.
///
/// # Mapping với SQL types
///
/// | SQL Type        | Value Variant     |
/// |-----------------|-------------------|
/// | INTEGER, BIGINT | `Integer(i64)`    |
/// | REAL, FLOAT     | `Float(f64)`      |
/// | TEXT, VARCHAR    | `Text(String)`    |
/// | BLOB, BYTEA     | `Blob(Vec<u8>)`   |
/// | NULL            | `None` (trong `Option<Value>`) |
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Giá trị số nguyên (INTEGER, BIGINT, SERIAL, ...)
    Integer(i64),
    /// Giá trị số thực (REAL, FLOAT, DOUBLE, ...)
    Float(f64),
    /// Giá trị chuỗi (TEXT, VARCHAR, CHAR, ...)
    Text(String),
    /// Giá trị nhị phân (BLOB, BYTEA, ...)
    Blob(Vec<u8>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(v) => write!(f, "{v}"),
            Value::Float(v) => write!(f, "{v}"),
            Value::Text(v) => write!(f, "{v}"),
            Value::Blob(v) => write!(f, "<{} bytes>", v.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_integer() {
        assert_eq!(Value::Integer(42).to_string(), "42");
        assert_eq!(Value::Integer(0).to_string(), "0");
        assert_eq!(Value::Integer(-1).to_string(), "-1");
        assert_eq!(Value::Integer(i64::MAX).to_string(), i64::MAX.to_string());
    }

    #[test]
    fn test_display_float() {
        assert_eq!(Value::Float(3.14).to_string(), "3.14");
        assert_eq!(Value::Float(0.0).to_string(), "0");
        assert_eq!(Value::Float(-1.5).to_string(), "-1.5");
    }

    #[test]
    fn test_display_text() {
        assert_eq!(Value::Text("hello".to_string()).to_string(), "hello");
        assert_eq!(Value::Text("".to_string()).to_string(), "");
        assert_eq!(
            Value::Text("with spaces".to_string()).to_string(),
            "with spaces"
        );
    }

    #[test]
    fn test_display_blob() {
        assert_eq!(Value::Blob(vec![1, 2, 3]).to_string(), "<3 bytes>");
        assert_eq!(Value::Blob(vec![]).to_string(), "<0 bytes>");
        assert_eq!(Value::Blob(vec![0; 1024]).to_string(), "<1024 bytes>");
    }

    #[test]
    fn test_clone() {
        let v = Value::Integer(42);
        assert_eq!(v.clone(), v);

        let v = Value::Text("hello".to_string());
        assert_eq!(v.clone(), v);
    }

    #[test]
    fn test_partial_eq() {
        assert_eq!(Value::Integer(1), Value::Integer(1));
        assert_ne!(Value::Integer(1), Value::Integer(2));
        assert_ne!(Value::Integer(1), Value::Float(1.0));
        assert_eq!(Value::Text("a".to_string()), Value::Text("a".to_string()));
        assert_ne!(Value::Text("a".to_string()), Value::Text("b".to_string()));
    }

    #[test]
    fn test_debug() {
        let v = Value::Integer(42);
        assert_eq!(format!("{:?}", v), "Integer(42)");
    }
}
