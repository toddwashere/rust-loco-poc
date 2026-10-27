#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20261027_215211_companies;
mod m20261027_215356_contacts;
mod m20261027_215509_deals;
mod m20261027_215624_notes;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20261027_215211_companies::Migration),
            Box::new(m20261027_215356_contacts::Migration),
            Box::new(m20261027_215509_deals::Migration),
            Box::new(m20261027_215624_notes::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
