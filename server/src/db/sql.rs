//! SQL built at runtime, for filters and the table and column names the code
//! picks. Text is `&'static str`, so only the code writes it, and every value
//! is bound. Fixed queries use the `sqlx::query!` macros instead.
//!
//! `clippy.toml` and `tests/sql_in_db.rs` keep `QueryBuilder` and `AssertSqlSafe`
//! to this module.

#![allow(clippy::disallowed_types)]

use sqlx::query::{Query, QueryAs, QueryScalar};
use sqlx::sqlite::{SqliteArguments, SqliteRow};
use sqlx::{Encode, FromRow, QueryBuilder, SqlStr, Sqlite, Type};

pub struct Sql(QueryBuilder<Sqlite>);

impl Sql {
    pub fn new(sql: &'static str) -> Self {
        Self(QueryBuilder::new(sql))
    }

    pub fn push(&mut self, sql: &'static str) -> &mut Self {
        self.0.push(sql);
        self
    }

    pub fn push_bind<'t, T>(&mut self, value: T) -> &mut Self
    where
        T: Encode<'t, Sqlite> + Type<Sqlite>,
    {
        self.0.push_bind(value);
        self
    }

    /// Pushes `(?, ?, ...)`, one bound value each.
    pub fn push_list<T>(&mut self, values: &[T]) -> &mut Self
    where
        T: Clone + for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
    {
        self.0.push("(");
        let mut list = self.0.separated(", ");
        for value in values {
            list.push_bind(value.clone());
        }
        list.push_unseparated(")");
        self
    }

    pub fn sql(&self) -> SqlStr {
        self.0.sql()
    }

    pub fn build(&mut self) -> Query<'_, Sqlite, SqliteArguments> {
        self.0.build()
    }

    pub fn build_query_as<'q, T>(&'q mut self) -> QueryAs<'q, Sqlite, T, SqliteArguments>
    where
        T: FromRow<'q, SqliteRow>,
    {
        self.0.build_query_as()
    }

    pub fn build_query_scalar<'q, T>(&'q mut self) -> QueryScalar<'q, Sqlite, T, SqliteArguments>
    where
        (T,): for<'r> FromRow<'r, SqliteRow>,
    {
        self.0.build_query_scalar()
    }
}
