use engine::{DatabaseConfig, QueryResult, SqlClient, TableKind, Value};

fn sqlite_memory_config() -> DatabaseConfig {
    DatabaseConfig::sqlite("sqlite::memory:")
}

#[tokio::test]
async fn test_full_crud_flow() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    // CREATE
    client
        .execute(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT UNIQUE,
                age INTEGER
            )",
        )
        .await
        .unwrap();

    // INSERT
    let insert1 = client
        .execute("INSERT INTO users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)")
        .await
        .unwrap();
    match insert1 {
        QueryResult::Execution {
            rows_affected,
            last_insert_rowid,
        } => {
            assert_eq!(rows_affected, 1);
            assert_eq!(last_insert_rowid, Some(1));
        }
        _ => panic!("Expected Execution for INSERT"),
    }

    let insert2 = client
        .execute("INSERT INTO users (name, email, age) VALUES ('Bob', 'bob@example.com', 25)")
        .await
        .unwrap();
    match insert2 {
        QueryResult::Execution {
            last_insert_rowid, ..
        } => {
            assert_eq!(last_insert_rowid, Some(2));
        }
        _ => panic!("Expected Execution for INSERT"),
    }

    // SELECT all
    let select_all = client
        .execute("SELECT id, name, email, age FROM users ORDER BY id")
        .await
        .unwrap();
    match select_all {
        QueryResult::Query { columns, rows } => {
            assert_eq!(columns.len(), 4);
            assert_eq!(columns[0].name, "id");
            assert_eq!(columns[1].name, "name");
            assert_eq!(columns[2].name, "email");
            assert_eq!(columns[3].name, "age");
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].values[1], Some(Value::Text("Alice".to_string())));
            assert_eq!(rows[1].values[1], Some(Value::Text("Bob".to_string())));
        }
        _ => panic!("Expected Query for SELECT"),
    }

    // SELECT with WHERE
    let select_where = client
        .execute("SELECT name FROM users WHERE age > 27")
        .await
        .unwrap();
    match select_where {
        QueryResult::Query { rows, .. } => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].values[0], Some(Value::Text("Alice".to_string())));
        }
        _ => panic!("Expected Query for SELECT WHERE"),
    }

    // UPDATE
    let update = client
        .execute("UPDATE users SET age = 31 WHERE name = 'Alice'")
        .await
        .unwrap();
    match update {
        QueryResult::Execution { rows_affected, .. } => {
            assert_eq!(rows_affected, 1);
        }
        _ => panic!("Expected Execution for UPDATE"),
    }

    // Verify UPDATE
    let verify_update = client
        .execute("SELECT age FROM users WHERE name = 'Alice'")
        .await
        .unwrap();
    match verify_update {
        QueryResult::Query { rows, .. } => {
            assert_eq!(rows[0].values[0], Some(Value::Integer(31)));
        }
        _ => panic!("Expected Query"),
    }

    // DELETE
    let delete = client
        .execute("DELETE FROM users WHERE name = 'Bob'")
        .await
        .unwrap();
    match delete {
        QueryResult::Execution { rows_affected, .. } => {
            assert_eq!(rows_affected, 1);
        }
        _ => panic!("Expected Execution for DELETE"),
    }

    // Verify DELETE
    let verify_delete = client
        .execute("SELECT COUNT(*) as cnt FROM users")
        .await
        .unwrap();
    match verify_delete {
        QueryResult::Query { rows, .. } => {
            assert_eq!(rows[0].values[0], Some(Value::Integer(1)));
        }
        _ => panic!("Expected Query"),
    }

    // DROP
    let drop = client.execute("DROP TABLE users").await.unwrap();
    match drop {
        QueryResult::Execution { .. } => {
            // DROP TABLE không có ý nghĩa về rows_affected
        }
        _ => panic!("Expected Execution for DROP"),
    }
}

#[tokio::test]
async fn test_multiple_tables() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    client
        .execute("CREATE TABLE categories (id INTEGER PRIMARY KEY, name TEXT)")
        .await
        .unwrap();
    client
        .execute("CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, category_id INTEGER)")
        .await
        .unwrap();

    client
        .execute("INSERT INTO categories VALUES (1, 'Electronics'), (2, 'Books')")
        .await
        .unwrap();
    client
        .execute("INSERT INTO products VALUES (1, 'Laptop', 1), (2, 'Novel', 2), (3, 'Phone', 1)")
        .await
        .unwrap();

    // JOIN query
    let join_result = client
        .execute(
            "SELECT p.name, c.name as category
             FROM products p
             JOIN categories c ON p.category_id = c.id
             ORDER BY p.id",
        )
        .await
        .unwrap();
    match join_result {
        QueryResult::Query { columns, rows } => {
            assert_eq!(columns.len(), 2);
            assert_eq!(rows.len(), 3);
            assert_eq!(rows[0].values[0], Some(Value::Text("Laptop".to_string())));
            assert_eq!(
                rows[0].values[1],
                Some(Value::Text("Electronics".to_string()))
            );
        }
        _ => panic!("Expected Query for JOIN"),
    }
}

#[tokio::test]
async fn test_aggregate_queries() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    client
        .execute("CREATE TABLE scores (id INTEGER PRIMARY KEY, name TEXT, score INTEGER)")
        .await
        .unwrap();
    client
        .execute("INSERT INTO scores (name, score) VALUES ('Alice', 90), ('Bob', 80), ('Carol', 95)")
        .await
        .unwrap();

    // COUNT
    let count = client
        .execute("SELECT COUNT(*) as cnt FROM scores")
        .await
        .unwrap();
    match count {
        QueryResult::Query { rows, .. } => {
            assert_eq!(rows[0].values[0], Some(Value::Integer(3)));
        }
        _ => panic!("Expected Query for COUNT"),
    }

    // SUM
    let sum = client
        .execute("SELECT SUM(score) as total FROM scores")
        .await
        .unwrap();
    match sum {
        QueryResult::Query { rows, .. } => {
            assert_eq!(rows[0].values[0], Some(Value::Integer(265)));
        }
        _ => panic!("Expected Query for SUM"),
    }

    // AVG
    let avg = client
        .execute("SELECT AVG(score) as avg_score FROM scores")
        .await
        .unwrap();
    match avg {
        QueryResult::Query { rows, .. } => {
            // AVG returns float in SQLite
            if let Some(Value::Float(v)) = rows[0].values[0] {
                assert!((v - 88.333).abs() < 0.01);
            } else {
                panic!("Expected Float for AVG");
            }
        }
        _ => panic!("Expected Query for AVG"),
    }

    // MAX, MIN
    let max_min = client
        .execute("SELECT MAX(score), MIN(score) FROM scores")
        .await
        .unwrap();
    match max_min {
        QueryResult::Query { rows, .. } => {
            assert_eq!(rows[0].values[0], Some(Value::Integer(95)));
            assert_eq!(rows[0].values[1], Some(Value::Integer(80)));
        }
        _ => panic!("Expected Query for MAX/MIN"),
    }

    // GROUP BY
    client
        .execute("CREATE TABLE sales (id INTEGER PRIMARY KEY, region TEXT, amount INTEGER)")
        .await
        .unwrap();
    client
        .execute("INSERT INTO sales (region, amount) VALUES ('North', 100), ('North', 200), ('South', 150)")
        .await
        .unwrap();

    let group_by = client
        .execute("SELECT region, SUM(amount) as total FROM sales GROUP BY region ORDER BY region")
        .await
        .unwrap();
    match group_by {
        QueryResult::Query { rows, .. } => {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].values[0], Some(Value::Text("North".to_string())));
            assert_eq!(rows[0].values[1], Some(Value::Integer(300)));
            assert_eq!(rows[1].values[0], Some(Value::Text("South".to_string())));
            assert_eq!(rows[1].values[1], Some(Value::Integer(150)));
        }
        _ => panic!("Expected Query for GROUP BY"),
    }
}

#[tokio::test]
async fn test_error_handling_syntax_error() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    let result = client.execute("SELCT * FROM nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_handling_table_not_found() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    let result = client.execute("SELECT * FROM nonexistent_table").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_handling_constraint_violation() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    client
        .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT UNIQUE)")
        .await
        .unwrap();
    client
        .execute("INSERT INTO test VALUES (1, 'Alice')")
        .await
        .unwrap();

    // UNIQUE constraint violation
    let result = client
        .execute("INSERT INTO test VALUES (2, 'Alice')")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_special_characters_in_text() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    client
        .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, val TEXT)")
        .await
        .unwrap();

    // Text with special characters
    client
        .execute("INSERT INTO test (val) VALUES ('hello world! @#$%^&*()')")
        .await
        .unwrap();

    let result = client.execute("SELECT val FROM test").await.unwrap();
    match result {
        QueryResult::Query { rows, .. } => {
            assert_eq!(
                rows[0].values[0],
                Some(Value::Text("hello world! @#$%^&*()".to_string()))
            );
        }
        _ => panic!("Expected Query"),
    }
}

#[tokio::test]
async fn test_large_insert_and_select() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    client
        .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, val INTEGER)")
        .await
        .unwrap();

    // Insert 100 rows
    for i in 1..=100 {
        client
            .execute(&format!("INSERT INTO test (val) VALUES ({i})"))
            .await
            .unwrap();
    }

    let result = client
        .execute("SELECT COUNT(*) FROM test")
        .await
        .unwrap();
    match result {
        QueryResult::Query { rows, .. } => {
            assert_eq!(rows[0].values[0], Some(Value::Integer(100)));
        }
        _ => panic!("Expected Query"),
    }
}

#[tokio::test]
async fn test_pragma_queries() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    client
        .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
        .await
        .unwrap();

    // PRAGMA table_info
    let result = client
        .execute("PRAGMA table_info(test)")
        .await
        .unwrap();
    match result {
        QueryResult::Query { columns, rows } => {
            assert_eq!(columns.len(), 6); // cid, name, type, notnull, dflt_value, pk
            assert_eq!(rows.len(), 2); // id, name
        }
        _ => panic!("Expected Query for PRAGMA"),
    }
}

#[tokio::test]
async fn test_cte_query() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    client
        .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, val INTEGER)")
        .await
        .unwrap();
    client
        .execute("INSERT INTO test VALUES (1, 10), (2, 20), (3, 30)")
        .await
        .unwrap();

    let result = client
        .execute("WITH cte AS (SELECT val * 2 as doubled FROM test) SELECT * FROM cte ORDER BY doubled")
        .await
        .unwrap();
    match result {
        QueryResult::Query { rows, .. } => {
            assert_eq!(rows.len(), 3);
            assert_eq!(rows[0].values[0], Some(Value::Integer(20)));
            assert_eq!(rows[1].values[0], Some(Value::Integer(40)));
            assert_eq!(rows[2].values[0], Some(Value::Integer(60)));
        }
        _ => panic!("Expected Query for CTE"),
    }
}

// ===== Schema Introspection Integration Tests =====

#[tokio::test]
async fn test_schema_full_flow() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    // Create multiple tables
    client
        .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT)")
        .await
        .unwrap();
    client
        .execute("CREATE TABLE posts (id INTEGER PRIMARY KEY, title TEXT, user_id INTEGER)")
        .await
        .unwrap();
    client
        .execute(
            "CREATE VIEW active_users AS SELECT * FROM users WHERE id > 0",
        )
        .await
        .unwrap();

    // list_tables should return both tables and view
    let tables = client.list_tables().await.unwrap();
    assert_eq!(tables.len(), 3);

    let table_names: Vec<&str> = tables
        .iter()
        .filter(|t| t.kind == TableKind::Table)
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(table_names.len(), 2);
    assert!(table_names.contains(&"users"));
    assert!(table_names.contains(&"posts"));

    let view_names: Vec<&str> = tables
        .iter()
        .filter(|t| t.kind == TableKind::View)
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(view_names.len(), 1);
    assert!(view_names.contains(&"active_users"));

    // get_table_info for users
    let users_info = client.get_table_info("users").await.unwrap();
    assert_eq!(users_info.name, "users");
    assert_eq!(users_info.columns.len(), 3);

    let id_col = &users_info.columns[0];
    assert_eq!(id_col.name, "id");
    assert_eq!(id_col.data_type, "INTEGER");
    assert!(id_col.is_primary_key);

    let name_col = &users_info.columns[1];
    assert_eq!(name_col.name, "name");
    assert_eq!(name_col.data_type, "TEXT");
    assert!(!name_col.nullable); // NOT NULL

    let email_col = &users_info.columns[2];
    assert_eq!(email_col.name, "email");
    assert!(email_col.nullable);

    assert_eq!(users_info.primary_key.columns, vec!["id"]);

    // get_table_row_count
    client
        .execute("INSERT INTO users (name) VALUES ('Alice'), ('Bob'), ('Carol')")
        .await
        .unwrap();
    let count = client.get_table_row_count("users").await.unwrap();
    assert_eq!(count, 3);

    // get_table_info for nonexistent table
    let result = client.get_table_info("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_schema_with_fk_and_indexes() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    client
        .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
        .await
        .unwrap();
    client
        .execute("CREATE TABLE posts (id INTEGER PRIMARY KEY, title TEXT, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE)")
        .await
        .unwrap();
    client
        .execute("CREATE INDEX idx_posts_user ON posts(user_id)")
        .await
        .unwrap();
    client
        .execute("CREATE INDEX idx_posts_title ON posts(title)")
        .await
        .unwrap();

    let posts_info = client.get_table_info("posts").await.unwrap();

    // Foreign keys
    assert_eq!(posts_info.foreign_keys.len(), 1);
    let fk = &posts_info.foreign_keys[0];
    assert_eq!(fk.columns, vec!["user_id"]);
    assert_eq!(fk.references_table, "users");
    assert_eq!(fk.references_columns, vec!["id"]);
    assert_eq!(fk.on_delete, Some("CASCADE".to_string()));

    // Indexes (user-created ones)
    let user_idx = posts_info
        .indexes
        .iter()
        .find(|i| i.name == "idx_posts_user");
    assert!(user_idx.is_some());
    assert!(user_idx.unwrap().columns.contains(&"user_id".to_string()));

    let title_idx = posts_info
        .indexes
        .iter()
        .find(|i| i.name == "idx_posts_title");
    assert!(title_idx.is_some());
    assert!(!title_idx.unwrap().is_unique); // Non-unique index
}

#[tokio::test]
async fn test_schema_views_only() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    client
        .execute("CREATE TABLE t1 (id INTEGER)")
        .await
        .unwrap();
    client
        .execute("CREATE VIEW v1 AS SELECT * FROM t1")
        .await
        .unwrap();
    client
        .execute("CREATE VIEW v2 AS SELECT id * 2 as doubled FROM t1")
        .await
        .unwrap();

    let views = client.list_views().await.unwrap();
    assert_eq!(views.len(), 2);
    assert!(views.contains(&"v1".to_string()));
    assert!(views.contains(&"v2".to_string()));
}

#[tokio::test]
async fn test_schema_composite_primary_key() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    client
        .execute("CREATE TABLE order_items (order_id INTEGER, product_id INTEGER, quantity INTEGER, PRIMARY KEY (order_id, product_id))")
        .await
        .unwrap();

    let info = client.get_table_info("order_items").await.unwrap();
    assert_eq!(info.primary_key.columns.len(), 2);
    assert!(info.primary_key.columns.contains(&"order_id".to_string()));
    assert!(info.primary_key.columns.contains(&"product_id".to_string()));

    // Columns that are PK should be marked
    let order_col = info.columns.iter().find(|c| c.name == "order_id").unwrap();
    assert!(order_col.is_primary_key);

    let product_col = info
        .columns
        .iter()
        .find(|c| c.name == "product_id")
        .unwrap();
    assert!(product_col.is_primary_key);

    let qty_col = info.columns.iter().find(|c| c.name == "quantity").unwrap();
    assert!(!qty_col.is_primary_key);
}

#[tokio::test]
async fn test_schema_default_values() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    client
        .execute("CREATE TABLE test (id INTEGER, status TEXT DEFAULT 'active', count INTEGER DEFAULT 0)")
        .await
        .unwrap();

    let info = client.get_table_info("test").await.unwrap();

    let status_col = info.columns.iter().find(|c| c.name == "status").unwrap();
    assert_eq!(status_col.default_value, Some("'active'".to_string()));

    let count_col = info.columns.iter().find(|c| c.name == "count").unwrap();
    assert_eq!(count_col.default_value, Some("0".to_string()));
}

#[tokio::test]
async fn test_schema_row_count_after_operations() {
    let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

    client
        .execute("CREATE TABLE test (id INTEGER PRIMARY KEY)")
        .await
        .unwrap();

    // Empty table
    assert_eq!(client.get_table_row_count("test").await.unwrap(), 0);

    // Insert 5 rows
    client
        .execute("INSERT INTO test VALUES (1), (2), (3), (4), (5)")
        .await
        .unwrap();
    assert_eq!(client.get_table_row_count("test").await.unwrap(), 5);

    // Delete 2 rows
    client
        .execute("DELETE FROM test WHERE id > 3")
        .await
        .unwrap();
    assert_eq!(client.get_table_row_count("test").await.unwrap(), 3);
}
