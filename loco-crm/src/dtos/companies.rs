use sea_orm::prelude::DateTimeWithTimeZone;
use ts_rs::TS;

#[derive(serde::Serialize, serde::Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct CompanyDto {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub domain: Option<String>,
    pub industry: Option<String>,
    #[ts(type = "string")]
    pub created_at: DateTimeWithTimeZone,
    #[ts(type = "string")]
    pub updated_at: DateTimeWithTimeZone,
}

impl From<crate::models::_entities::companies::Model> for CompanyDto {
    fn from(m: crate::models::_entities::companies::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            domain: m.domain,
            industry: m.industry,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct CreateCompany {
    pub name: String,
    pub domain: Option<String>,
    pub industry: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct UpdateCompany {
    pub name: String,
    pub domain: Option<String>,
    pub industry: Option<String>,
}
