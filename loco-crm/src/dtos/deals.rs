use sea_orm::prelude::DateTimeWithTimeZone;
use ts_rs::TS;

#[derive(serde::Serialize, serde::Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct DealDto {
    #[ts(type = "number")]
    pub id: i64,
    pub title: String,
    #[ts(type = "number | null")]
    pub amount_cents: Option<i64>,
    pub stage: String,
    #[ts(type = "number | null")]
    pub company_id: Option<i64>,
    #[ts(type = "number | null")]
    pub contact_id: Option<i64>,
    #[ts(type = "string")]
    pub created_at: DateTimeWithTimeZone,
    #[ts(type = "string")]
    pub updated_at: DateTimeWithTimeZone,
}

impl From<crate::models::_entities::deals::Model> for DealDto {
    fn from(m: crate::models::_entities::deals::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            amount_cents: m.amount_cents,
            stage: m.stage,
            company_id: m.company_id,
            contact_id: m.contact_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct CreateDeal {
    pub title: String,
    #[ts(type = "number | null")]
    pub amount_cents: Option<i64>,
    pub stage: String,
    #[ts(type = "number | null")]
    pub company_id: Option<i64>,
    #[ts(type = "number | null")]
    pub contact_id: Option<i64>,
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct UpdateDeal {
    pub title: String,
    #[ts(type = "number | null")]
    pub amount_cents: Option<i64>,
    pub stage: String,
    #[ts(type = "number | null")]
    pub company_id: Option<i64>,
    #[ts(type = "number | null")]
    pub contact_id: Option<i64>,
}
