pub use super::_entities::deals::{ActiveModel, Column, Entity, Model};
use super::_entities::{companies, contacts};
use loco_rs::prelude::*;
use sea_orm::QuerySelect;
use serde::{Deserialize, Serialize};
pub type Deals = Entity;

/// Pipeline stages, in board order.
pub const STAGES: [&str; 5] = ["lead", "qualified", "proposal", "won", "lost"];

fn validate_stage(stage: &str) -> Result<(), validator::ValidationError> {
    if STAGES.contains(&stage) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("stage").with_message("Unknown pipeline stage.".into()))
    }
}

#[derive(Debug, Validate, Deserialize)]
pub struct Validator {
    #[validate(length(min = 1, message = "Deal title is required."))]
    pub title: String,
    #[validate(custom(function = "validate_stage"))]
    pub stage: String,
}

impl Validatable for ActiveModel {
    fn validator(&self) -> Box<dyn Validate> {
        Box::new(Validator {
            title: self.title.as_ref().to_owned(),
            stage: self.stage.as_ref().to_owned(),
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

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct StageSummary {
    pub stage: String,
    pub count: i64,
    pub total_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct DealCard {
    pub deal: Model,
    pub company: Option<companies::Model>,
}

#[derive(Debug, Serialize)]
pub struct BoardColumn {
    pub stage: &'static str,
    pub cards: Vec<DealCard>,
    pub total_cents: i64,
}

impl Model {
    /// Count and value per stage, aggregated in SQL, returned in board order
    /// (stages with no deals included as zero).
    ///
    /// # Errors
    /// On database errors.
    pub async fn pipeline_summary(db: &DatabaseConnection) -> ModelResult<Vec<StageSummary>> {
        let rows: Vec<(String, i64, Option<i64>)> = Entity::find()
            .select_only()
            .column(Column::Stage)
            .column_as(Column::Id.count(), "count")
            .column_as(Column::AmountCents.sum(), "total")
            .group_by(Column::Stage)
            .into_tuple()
            .all(db)
            .await?;
        Ok(STAGES
            .iter()
            .map(|stage| {
                let row = rows.iter().find(|(s, _, _)| s == stage);
                StageSummary {
                    stage: (*stage).to_string(),
                    count: row.map_or(0, |r| r.1),
                    total_cents: row.and_then(|r| r.2).unwrap_or(0),
                }
            })
            .collect())
    }

    /// The kanban board: open deals grouped by stage, newest first, capped per
    /// column so the page stays fast with a large pipeline.
    ///
    /// # Errors
    /// On database errors.
    pub async fn board(db: &DatabaseConnection, per_column: u64) -> ModelResult<Vec<BoardColumn>> {
        let summary = Self::pipeline_summary(db).await?;
        let mut columns = Vec::with_capacity(STAGES.len());
        for (stage, sum) in STAGES.iter().zip(summary) {
            let cards = Entity::find()
                .find_also_related(companies::Entity)
                .filter(Column::Stage.eq(*stage))
                .order_by_desc(Column::UpdatedAt)
                .limit(per_column)
                .all(db)
                .await?
                .into_iter()
                .map(|(deal, company)| DealCard { deal, company })
                .collect();
            columns.push(BoardColumn {
                stage,
                cards,
                total_cents: sum.total_cents,
            });
        }
        Ok(columns)
    }

    /// # Errors
    /// `EntityNotFound` if no deal has this id.
    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> ModelResult<Self> {
        Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    /// # Errors
    /// On database errors.
    pub async fn contact(&self, db: &DatabaseConnection) -> ModelResult<Option<contacts::Model>> {
        Ok(self.find_related(contacts::Entity).one(db).await?)
    }
}

impl ActiveModel {
    /// Move a deal to another stage. Validation rejects unknown stages.
    ///
    /// # Errors
    /// `ModelError::Validation` for an unknown stage.
    pub async fn move_to_stage(
        mut self,
        db: &DatabaseConnection,
        stage: &str,
    ) -> ModelResult<Model> {
        self.stage = Set(stage.to_string());
        Ok(self.update(db).await?)
    }
}
