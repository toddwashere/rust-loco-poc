pub use super::_entities::companies::{ActiveModel, Column, Entity, Model};
use loco_rs::prelude::*;
use serde::Deserialize;
pub type Companies = Entity;

#[derive(Debug, Validate, Deserialize)]
pub struct Validator {
    #[validate(length(min = 1, message = "Company name is required."))]
    pub name: String,
}

impl Validatable for ActiveModel {
    fn validator(&self) -> Box<dyn Validate> {
        Box::new(Validator {
            name: self.name.as_ref().to_owned(),
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

impl Model {
    /// All companies, alphabetical — for `<select>` options.
    ///
    /// # Errors
    /// On database errors.
    pub async fn all_by_name(db: &DatabaseConnection) -> ModelResult<Vec<Self>> {
        Ok(Entity::find().order_by_asc(Column::Name).all(db).await?)
    }
}
