//! Reactive data layer with SQLite persistence.
//!
//! Provides the [`Model`] trait (derivable), [`Handle`] for typed references
//! to persisted rows, reactive queries via [`query_all`], and schema
//! migrations.
//!
//! # Example
//! ```ignore
//! use lumen_data::{Model, open, register, insert, query_all};
//! use lumen_core::reactive::get;
//!
//! #[derive(Clone, Model)]
//! struct Todo {
//!     id: i64,
//!     title: String,
//!     done: bool,
//! }
//!
//! open(":memory:").unwrap();
//! register::<Todo>().unwrap();
//! insert(Todo { id: 0, title: "Test".into(), done: false }).unwrap();
//! let todos = query_all::<Todo>();
//! assert_eq!(get(todos).len(), 1);
//! ```

use std::cell::RefCell;
use std::marker::PhantomData;

use lumen_core::reactive::{create_memo, create_signal, get, set, Signal};
use rusqlite::Connection;

pub use lumen_data_derive::Model;

// ── Value types ──────────────────────────────────────────────────────────

/// A database-compatible value.
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Bool(bool),
}

/// SQL column type.
#[derive(Debug, Clone, Copy)]
pub enum ColumnType {
    Integer,
    Real,
    Text,
    Bool,
}

/// Column definition for a model field.
#[derive(Debug, Clone, Copy)]
pub struct ColumnDef {
    pub name: &'static str,
    pub ty: ColumnType,
    pub primary_key: bool,
}

// ── Model trait ──────────────────────────────────────────────────────────

/// A persistable data model.
///
/// Derive with `#[derive(Model)]` on a struct whose first field is `id: i64`.
pub trait Model: Clone + PartialEq + 'static {
    /// SQL table name (snake_case, pluralized).
    fn table_name() -> &'static str;
    /// Column definitions in field order.
    fn columns() -> &'static [ColumnDef];
    /// Primary key value.
    fn id(&self) -> i64;
    /// Set the primary key (called after insert).
    fn set_id(&mut self, id: i64);
    /// Non-id field values for INSERT/UPDATE params.
    fn to_params(&self) -> Vec<Value>;
    /// Reconstruct from a full row (all columns including id).
    fn from_values(values: &[Value]) -> Self;
}

// ── Handle ───────────────────────────────────────────────────────────────

/// A typed, `Copy` reference to a persisted model row.
///
/// Use with [`find`], [`save`], and [`delete`].
#[derive(Debug)]
pub struct Handle<T: Model> {
    id: i64,
    _marker: PhantomData<T>,
}

impl<T: Model> Handle<T> {
    /// Create a handle from a known primary key.
    pub fn from_id(id: i64) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    /// The row's primary key.
    pub fn id(&self) -> i64 {
        self.id
    }
}

impl<T: Model> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Model> Copy for Handle<T> {}

// ── Error ────────────────────────────────────────────────────────────────

/// Data layer error.
#[derive(Debug)]
pub enum DataError {
    /// SQLite error.
    Sql(rusqlite::Error),
    /// Store not opened.
    NotOpen,
}

impl From<rusqlite::Error> for DataError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sql(e)
    }
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sql(e) => write!(f, "sqlite: {e}"),
            Self::NotOpen => write!(f, "store not opened"),
        }
    }
}

impl std::error::Error for DataError {}

// ── Migration ────────────────────────────────────────────────────────────

/// A schema migration step.
pub struct Migration {
    /// Monotonically increasing version number.
    pub version: u32,
    /// SQL to execute for this migration.
    pub sql: &'static str,
}

// ── Store (thread-local) ─────────────────────────────────────────────────

struct StoreInner {
    conn: Connection,
    version: Signal<u64>,
}

thread_local! {
    static STORE: RefCell<Option<StoreInner>> = const { RefCell::new(None) };
}

fn with_store<R>(f: impl FnOnce(&StoreInner) -> Result<R, DataError>) -> Result<R, DataError> {
    STORE.with(|cell| {
        let borrow = cell.borrow();
        let store = borrow.as_ref().ok_or(DataError::NotOpen)?;
        f(store)
    })
}

/// Open (or create) a SQLite database at `path`.
///
/// Use `":memory:"` for an in-memory database.
pub fn open(path: &str) -> Result<(), DataError> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    let version = create_signal(0_u64);
    STORE.with(|cell| {
        *cell.borrow_mut() = Some(StoreInner { conn, version });
    });
    Ok(())
}

/// Register a model's table (CREATE TABLE IF NOT EXISTS).
pub fn register<M: Model>() -> Result<(), DataError> {
    with_store(|store| {
        let sql = create_table_sql::<M>();
        store.conn.execute_batch(&sql)?;
        Ok(())
    })
}

/// Apply pending migrations.
///
/// Creates a `_migrations` table to track applied versions.
pub fn migrate(migrations: &[Migration]) -> Result<(), DataError> {
    with_store(|store| {
        store.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (version INTEGER PRIMARY KEY)",
        )?;

        for m in migrations {
            let applied: bool = store.conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM _migrations WHERE version = ?1)",
                [m.version],
                |row| row.get(0),
            )?;
            if !applied {
                store.conn.execute_batch(m.sql)?;
                store
                    .conn
                    .execute("INSERT INTO _migrations (version) VALUES (?1)", [m.version])?;
            }
        }
        Ok(())
    })
}

/// Insert a model and return a [`Handle`] to it.
///
/// The model's `id` field is ignored; the database assigns one.
pub fn insert<M: Model>(model: M) -> Result<Handle<M>, DataError> {
    let (handle, ver_sig, new_ver) = with_store(|store| {
        let cols = non_id_columns::<M>();
        let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("?{i}")).collect();
        let col_names: Vec<&str> = cols.iter().map(|c| c.name).collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            M::table_name(),
            col_names.join(", "),
            placeholders.join(", ")
        );

        let params = model.to_params();
        let refs = params_to_sql(&params);
        store.conn.execute(&sql, refs.as_slice())?;
        let new_id = store.conn.last_insert_rowid();

        let v = get(store.version);
        Ok((
            Handle {
                id: new_id,
                _marker: PhantomData,
            },
            store.version,
            v + 1,
        ))
    })?;
    set(ver_sig, new_ver);
    Ok(handle)
}

/// Read a model by handle.
pub fn find<M: Model>(handle: Handle<M>) -> Result<Option<M>, DataError> {
    with_store(|store| {
        let col_names = all_column_names::<M>();
        let sql = format!(
            "SELECT {} FROM {} WHERE id = ?1",
            col_names.join(", "),
            M::table_name()
        );
        let result = store
            .conn
            .query_row(&sql, [handle.id], |row| Ok(row_to_model::<M>(row)))
            .optional()?;
        Ok(result)
    })
}

/// Update an existing row.
pub fn save<M: Model>(handle: Handle<M>, model: &M) -> Result<(), DataError> {
    let (ver_sig, new_ver) = with_store(|store| {
        let cols = non_id_columns::<M>();
        let sets: Vec<String> = cols
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{} = ?{}", c.name, i + 1))
            .collect();

        let sql = format!(
            "UPDATE {} SET {} WHERE id = ?{}",
            M::table_name(),
            sets.join(", "),
            cols.len() + 1
        );

        let mut params = model.to_params();
        params.push(Value::Integer(handle.id));
        let refs = params_to_sql(&params);
        store.conn.execute(&sql, refs.as_slice())?;

        let v = get(store.version);
        Ok((store.version, v + 1))
    })?;
    set(ver_sig, new_ver);
    Ok(())
}

/// Delete a row by handle.
pub fn delete<M: Model>(handle: Handle<M>) -> Result<(), DataError> {
    let (ver_sig, new_ver) = with_store(|store| {
        let sql = format!("DELETE FROM {} WHERE id = ?1", M::table_name());
        store.conn.execute(&sql, [handle.id])?;
        let v = get(store.version);
        Ok((store.version, v + 1))
    })?;
    set(ver_sig, new_ver);
    Ok(())
}

/// Reactive query: returns a [`Signal`] of all rows that updates on writes.
///
/// The signal re-reads from SQLite whenever the store changes.
pub fn query_all<M: Model>() -> Signal<Vec<M>> {
    let ver_sig = with_store(|store| Ok(store.version)).expect("store not open");
    create_memo(move || {
        let _ = get(ver_sig);
        with_store(|store| {
            let col_names = all_column_names::<M>();
            let sql = format!("SELECT {} FROM {}", col_names.join(", "), M::table_name());
            let mut stmt = store.conn.prepare(&sql)?;
            let rows = stmt.query_map([], |row| Ok(row_to_model::<M>(row)))?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
    })
}

/// Close the store and release the database connection.
pub fn close() {
    STORE.with(|cell| {
        cell.borrow_mut().take();
    });
}

/// Reset the store (for tests).
pub fn reset() {
    close();
    lumen_core::reactive::reset();
}

// ── Internal helpers ─────────────────────────────────────────────────────

fn create_table_sql<M: Model>() -> String {
    let cols: Vec<String> = M::columns()
        .iter()
        .map(|c| {
            let ty = match c.ty {
                ColumnType::Integer => "INTEGER",
                ColumnType::Real => "REAL",
                ColumnType::Text => "TEXT",
                ColumnType::Bool => "INTEGER",
            };
            if c.primary_key {
                format!("{} {} PRIMARY KEY AUTOINCREMENT", c.name, ty)
            } else {
                format!("{} {} NOT NULL", c.name, ty)
            }
        })
        .collect();

    format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        M::table_name(),
        cols.join(", ")
    )
}

fn non_id_columns<M: Model>() -> Vec<&'static ColumnDef> {
    M::columns().iter().filter(|c| !c.primary_key).collect()
}

fn all_column_names<M: Model>() -> Vec<&'static str> {
    M::columns().iter().map(|c| c.name).collect()
}

fn params_to_sql(values: &[Value]) -> Vec<&dyn rusqlite::types::ToSql> {
    values
        .iter()
        .map(|v| v as &dyn rusqlite::types::ToSql)
        .collect()
}

impl rusqlite::types::ToSql for Value {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            Value::Null => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Null,
            )),
            Value::Integer(i) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(*i),
            )),
            Value::Real(r) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Real(*r),
            )),
            Value::Text(s) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Text(s.clone()),
            )),
            Value::Bool(b) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(i64::from(*b)),
            )),
        }
    }
}

fn row_to_model<M: Model>(row: &rusqlite::Row<'_>) -> M {
    let cols = M::columns();
    let values: Vec<Value> = cols
        .iter()
        .enumerate()
        .map(|(i, c)| match c.ty {
            ColumnType::Integer => Value::Integer(row.get::<_, i64>(i).unwrap_or(0)),
            ColumnType::Real => Value::Real(row.get::<_, f64>(i).unwrap_or(0.0)),
            ColumnType::Text => Value::Text(row.get::<_, String>(i).unwrap_or_default()),
            ColumnType::Bool => Value::Bool(row.get::<_, i64>(i).unwrap_or(0) != 0),
        })
        .collect();
    M::from_values(&values)
}

use rusqlite::OptionalExtension;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Task {
        id: i64,
        title: String,
        done: bool,
    }

    impl Model for Task {
        fn table_name() -> &'static str {
            "tasks"
        }
        fn columns() -> &'static [ColumnDef] {
            static COLS: [ColumnDef; 3] = [
                ColumnDef {
                    name: "id",
                    ty: ColumnType::Integer,
                    primary_key: true,
                },
                ColumnDef {
                    name: "title",
                    ty: ColumnType::Text,
                    primary_key: false,
                },
                ColumnDef {
                    name: "done",
                    ty: ColumnType::Bool,
                    primary_key: false,
                },
            ];
            &COLS
        }
        fn id(&self) -> i64 {
            self.id
        }
        fn set_id(&mut self, id: i64) {
            self.id = id;
        }
        fn to_params(&self) -> Vec<Value> {
            vec![Value::Text(self.title.clone()), Value::Bool(self.done)]
        }
        fn from_values(values: &[Value]) -> Self {
            Self {
                id: match &values[0] {
                    Value::Integer(v) => *v,
                    _ => 0,
                },
                title: match &values[1] {
                    Value::Text(v) => v.clone(),
                    _ => String::new(),
                },
                done: match &values[2] {
                    Value::Bool(v) => *v,
                    _ => false,
                },
            }
        }
    }

    #[test]
    fn insert_and_query() {
        reset();
        open(":memory:").unwrap();
        register::<Task>().unwrap();

        insert(Task {
            id: 0,
            title: "Buy milk".into(),
            done: false,
        })
        .unwrap();

        insert(Task {
            id: 0,
            title: "Walk dog".into(),
            done: true,
        })
        .unwrap();

        let all = query_all::<Task>();
        let items = get(all);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Buy milk");
        assert!(!items[0].done);
        assert_eq!(items[1].title, "Walk dog");
        assert!(items[1].done);
    }

    #[test]
    fn find_and_save() {
        reset();
        open(":memory:").unwrap();
        register::<Task>().unwrap();

        let h = insert(Task {
            id: 0,
            title: "Original".into(),
            done: false,
        })
        .unwrap();

        let mut task = find(h).unwrap().unwrap();
        assert_eq!(task.title, "Original");

        task.title = "Updated".into();
        task.done = true;
        save(h, &task).unwrap();

        let reloaded = find(h).unwrap().unwrap();
        assert_eq!(reloaded.title, "Updated");
        assert!(reloaded.done);
    }

    #[test]
    fn delete_removes_row() {
        reset();
        open(":memory:").unwrap();
        register::<Task>().unwrap();

        let h = insert(Task {
            id: 0,
            title: "Temp".into(),
            done: false,
        })
        .unwrap();

        assert!(find(h).unwrap().is_some());
        delete(h).unwrap();
        assert!(find(h).unwrap().is_none());
    }

    #[test]
    fn reactive_query_updates() {
        reset();
        open(":memory:").unwrap();
        register::<Task>().unwrap();

        let all = query_all::<Task>();
        assert_eq!(get(all).len(), 0);

        insert(Task {
            id: 0,
            title: "One".into(),
            done: false,
        })
        .unwrap();

        assert_eq!(get(all).len(), 1);

        insert(Task {
            id: 0,
            title: "Two".into(),
            done: false,
        })
        .unwrap();

        assert_eq!(get(all).len(), 2);
    }

    #[test]
    fn migrations() {
        reset();
        open(":memory:").unwrap();
        register::<Task>().unwrap();

        migrate(&[Migration {
            version: 1,
            sql: "ALTER TABLE tasks ADD COLUMN priority INTEGER NOT NULL DEFAULT 0",
        }])
        .unwrap();

        // Re-running is idempotent
        migrate(&[Migration {
            version: 1,
            sql: "ALTER TABLE tasks ADD COLUMN priority INTEGER NOT NULL DEFAULT 0",
        }])
        .unwrap();
    }
}
