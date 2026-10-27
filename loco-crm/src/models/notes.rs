use super::_entities::contacts;
pub use super::_entities::notes::{ActiveModel, Column, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::QuerySelect;
use serde::{Deserialize, Serialize};
pub type Notes = Entity;

#[derive(Debug, Validate, Deserialize)]
pub struct Validator {
    #[validate(length(
        min = 1,
        max = 5000,
        message = "A note needs between 1 and 5000 characters."
    ))]
    pub body: String,
}

impl Validatable for ActiveModel {
    fn validator(&self) -> Box<dyn Validate> {
        Box::new(Validator {
            body: self.body.as_ref().trim().to_owned(),
        })
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        self.validate()?;
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RecentNote {
    pub note: Model,
    pub contact: Option<contacts::Model>,
}

impl Model {
    /// # Errors
    /// `ModelError::Validation` when the body is empty or too long.
    pub async fn create_for_contact(
        db: &DatabaseConnection,
        contact_id: i64,
        body: &str,
    ) -> ModelResult<Self> {
        Ok(ActiveModel {
            body: Set(body.trim().to_string()),
            contact_id: Set(contact_id),
            ..Default::default()
        }
        .insert(db)
        .await?)
    }

    /// Latest notes across all contacts, for the dashboard feed.
    ///
    /// # Errors
    /// On database errors.
    pub async fn recent(db: &DatabaseConnection, limit: u64) -> ModelResult<Vec<RecentNote>> {
        Ok(Entity::find()
            .find_also_related(contacts::Entity)
            .order_by_desc(Column::Id)
            .limit(limit)
            .all(db)
            .await?
            .into_iter()
            .map(|(note, contact)| RecentNote { note, contact })
            .collect())
    }
}
