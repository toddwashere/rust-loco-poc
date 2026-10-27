use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "contacts",
            &[
                ("id", ColType::PkAuto),
                ("first_name", ColType::String),
                ("last_name", ColType::String),
                ("email", ColType::String),
                ("phone", ColType::StringNull),
                ("title", ColType::StringNull),
                ("ai_summary", ColType::TextNull),
                ("avatar_key", ColType::StringNull),
            ],
            &[("company?", "")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "contacts").await
    }
}
