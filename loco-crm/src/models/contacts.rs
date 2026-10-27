pub use super::_entities::contacts::{ActiveModel, Column, Entity, Model};
use super::_entities::{companies, deals, notes};
use loco_rs::prelude::*;
use sea_orm::{Condition, QuerySelect};
use serde::{Deserialize, Serialize};
pub type Contacts = Entity;

#[derive(Debug, Validate, Deserialize)]
pub struct Validator {
    #[validate(length(min = 1, message = "First name is required."))]
    pub first_name: String,
    #[validate(email(message = "Enter a valid email address."))]
    pub email: String,
}

impl Validatable for ActiveModel {
    fn validator(&self) -> Box<dyn Validate> {
        Box::new(Validator {
            first_name: self.first_name.as_ref().to_owned(),
            email: self.email.as_ref().to_owned(),
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

/// Fields accepted from the server-rendered "new contact" form.
#[derive(Debug, Deserialize)]
pub struct NewContact {
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub title: Option<String>,
    pub company_id: Option<String>,
}

/// One row of the contact list: the contact plus its company, if any.
#[derive(Debug, Serialize)]
pub struct ContactRow {
    #[serde(flatten)]
    pub contact: Model,
    pub company: Option<companies::Model>,
}

/// Everything the contact detail page shows.
#[derive(Debug, Serialize)]
pub struct ContactDetails {
    pub contact: Model,
    pub company: Option<companies::Model>,
    pub notes: Vec<notes::Model>,
    pub deals: Vec<deals::Model>,
}

impl Model {
    /// Paged, optionally filtered list with companies joined in. Returns the
    /// rows and the total number of pages.
    ///
    /// # Errors
    /// On database errors.
    pub async fn search(
        db: &DatabaseConnection,
        q: Option<&str>,
        page: u64,
        per_page: u64,
    ) -> ModelResult<(Vec<ContactRow>, u64)> {
        let mut select = Entity::find()
            .find_also_related(companies::Entity)
            .order_by_desc(Column::Id);
        if let Some(q) = q.map(str::trim).filter(|q| !q.is_empty()) {
            let like = format!("%{q}%");
            select = select.filter(
                Condition::any()
                    .add(Column::FirstName.like(&like))
                    .add(Column::LastName.like(&like))
                    .add(Column::Email.like(&like)),
            );
        }
        let paginator = select.paginate(db, per_page);
        let pages = paginator.num_pages().await?;
        let rows = paginator
            .fetch_page(page.saturating_sub(1))
            .await?
            .into_iter()
            .map(|(contact, company)| ContactRow { contact, company })
            .collect();
        Ok((rows, pages))
    }

    /// One page with companies joined, no count query. Mirrors the query the
    /// Next.js twin runs so the benchmark compares like for like.
    ///
    /// # Errors
    /// On database errors.
    pub async fn page_with_company(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
    ) -> ModelResult<Vec<ContactRow>> {
        Ok(Entity::find()
            .find_also_related(companies::Entity)
            .order_by_desc(Column::Id)
            .limit(per_page)
            .offset(page.saturating_sub(1) * per_page)
            .all(db)
            .await?
            .into_iter()
            .map(|(contact, company)| ContactRow { contact, company })
            .collect())
    }

    /// # Errors
    /// `EntityNotFound` if no contact has this id.
    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> ModelResult<Self> {
        Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    /// # Errors
    /// `EntityNotFound` if no contact has this id.
    pub async fn find_details(db: &DatabaseConnection, id: i64) -> ModelResult<ContactDetails> {
        let contact = Self::find_by_id(db, id).await?;
        let company = contact.find_related(companies::Entity).one(db).await?;
        let notes = contact
            .find_related(notes::Entity)
            .order_by_desc(notes::Column::Id)
            .all(db)
            .await?;
        let deals = contact
            .find_related(deals::Entity)
            .order_by_desc(deals::Column::Id)
            .all(db)
            .await?;
        Ok(ContactDetails {
            contact,
            company,
            notes,
            deals,
        })
    }

    #[must_use]
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
            .trim()
            .to_string()
    }

    /// Create from the HTML form; empty optional fields become NULL.
    ///
    /// # Errors
    /// `ModelError::Validation` when the form is invalid.
    pub async fn create_from_form(db: &DatabaseConnection, form: NewContact) -> ModelResult<Self> {
        let non_empty =
            |s: Option<String>| s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        Ok(ActiveModel {
            first_name: Set(form.first_name.trim().to_string()),
            last_name: Set(form.last_name.trim().to_string()),
            email: Set(form.email.trim().to_string()),
            phone: Set(non_empty(form.phone)),
            title: Set(non_empty(form.title)),
            company_id: Set(non_empty(form.company_id).and_then(|id| id.parse().ok())),
            ..Default::default()
        }
        .insert(db)
        .await?)
    }

    /// Plain-text dossier of the contact and their notes, used as LLM input.
    ///
    /// # Errors
    /// On database errors.
    pub async fn dossier(&self, db: &DatabaseConnection) -> ModelResult<String> {
        let details = Self::find_details(db, self.id).await?;
        let mut out = format!(
            "Contact: {}\nTitle: {}\nEmail: {}\nCompany: {}\n\nDeals:\n",
            self.full_name(),
            self.title.as_deref().unwrap_or("-"),
            self.email,
            details.company.as_ref().map_or("-", |c| c.name.as_str()),
        );
        for d in &details.deals {
            out.push_str(&format!(
                "- {} ({}, ${})\n",
                d.title,
                d.stage,
                d.amount_cents.unwrap_or(0) / 100
            ));
        }
        out.push_str("\nNotes (newest first):\n");
        for n in &details.notes {
            out.push_str(&format!(
                "- [{}] {}\n",
                n.created_at.format("%Y-%m-%d"),
                n.body
            ));
        }
        Ok(out)
    }
}

impl ActiveModel {
    /// # Errors
    /// On database errors.
    pub async fn set_ai_summary(
        mut self,
        db: &DatabaseConnection,
        summary: Option<String>,
    ) -> ModelResult<Model> {
        self.ai_summary = Set(summary);
        Ok(self.update(db).await?)
    }

    /// # Errors
    /// On database errors.
    pub async fn set_avatar(mut self, db: &DatabaseConnection, key: String) -> ModelResult<Model> {
        self.avatar_key = Set(Some(key));
        Ok(self.update(db).await?)
    }
}
