use rusqlite_migration::{Migrations, M};
use std::cell::LazyCell;

// Define migrations. These are applied atomically.
pub const MIGRATIONS: LazyCell<Migrations> = LazyCell::new(|| {
    Migrations::new(vec![M::up(include_str!("./sql_migrations/202409211652_create_record.sql"))])
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_test() {
        assert!(MIGRATIONS.validate().is_ok());
    }
}
