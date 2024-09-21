use lazy_static::lazy_static;
use rusqlite_migration::{Migrations, M};

// Define migrations. These are applied atomically.
lazy_static! {
    static ref MIGRATIONS: Migrations<'static> = Migrations::new(vec![M::up(include_str!(
        "./sql_migrations/202409211652_create_record.sql"
    )),]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_test() {
        assert!(MIGRATIONS.validate().is_ok());
    }
}
