use sea_orm::prelude::DateTimeWithTimeZone;
use ts_rs::TS;

#[derive(serde::Serialize, serde::Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct ContactDto {
    #[ts(type = "number")]
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub title: Option<String>,
    pub ai_summary: Option<String>,
    pub avatar_key: Option<String>,
    #[ts(type = "number | null")]
    pub company_id: Option<i64>,
    #[ts(type = "string")]
    pub created_at: DateTimeWithTimeZone,
    #[ts(type = "string")]
    pub updated_at: DateTimeWithTimeZone,
}

impl From<crate::models::_entities::contacts::Model> for ContactDto {
    fn from(m: crate::models::_entities::contacts::Model) -> Self {
        Self {
            id: m.id,
            first_name: m.first_name,
            last_name: m.last_name,
            email: m.email,
            phone: m.phone,
            title: m.title,
            ai_summary: m.ai_summary,
            avatar_key: m.avatar_key,
            company_id: m.company_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct CreateContact {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub title: Option<String>,
    pub ai_summary: Option<String>,
    pub avatar_key: Option<String>,
    #[ts(type = "number | null")]
    pub company_id: Option<i64>,
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct UpdateContact {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub title: Option<String>,
    pub ai_summary: Option<String>,
    pub avatar_key: Option<String>,
    #[ts(type = "number | null")]
    pub company_id: Option<i64>,
}
