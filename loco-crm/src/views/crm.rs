//! Display shapes for the server-rendered CRM pages. Tera 2 ships no date or
//! currency filters, so formatting happens here, in Rust, once.
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::Serialize;

use crate::models::{
    _entities::{companies, contacts, deals, notes, users},
    contacts::ContactRow,
    deals::{BoardColumn, StageSummary},
    notes::RecentNote,
};

/// Human-readable field messages; `ModelError::Validation`'s Display is only
/// "Model validation failed".
#[must_use]
pub fn validation_messages(e: &loco_rs::validation::ModelValidationErrors) -> String {
    e.errors
        .iter()
        .flat_map(|(field, errs)| {
            errs.iter().map(move |err| err.message.clone().unwrap_or_else(|| format!("{field} is invalid")))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[must_use]
pub fn money(cents: Option<i64>) -> String {
    let dollars = cents.unwrap_or(0) / 100;
    let digits = dollars.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + 4);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if dollars < 0 {
        format!("-${out}")
    } else {
        format!("${out}")
    }
}

#[must_use]
pub fn when(t: &DateTimeWithTimeZone) -> String {
    t.format("%b %-d, %Y %-I:%M %p").to_string()
}

#[derive(Serialize)]
pub struct UserView {
    pub name: String,
    pub email: String,
}

impl From<&users::Model> for UserView {
    fn from(u: &users::Model) -> Self {
        Self {
            name: u.name.clone(),
            email: u.email.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct ContactView {
    pub id: i64,
    pub name: String,
    pub first_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub title: Option<String>,
    pub company: Option<String>,
    pub company_domain: Option<String>,
    pub ai_summary: Option<String>,
    pub avatar_url: Option<String>,
    pub created: String,
}

impl ContactView {
    #[must_use]
    pub fn new(c: &contacts::Model, company: Option<&companies::Model>) -> Self {
        Self {
            id: c.id,
            name: c.full_name(),
            first_name: c.first_name.clone(),
            email: c.email.clone(),
            phone: c.phone.clone(),
            title: c.title.clone(),
            company: company.map(|co| co.name.clone()),
            company_domain: company.and_then(|co| co.domain.clone()),
            ai_summary: c.ai_summary.clone(),
            avatar_url: c.avatar_key.as_ref().map(|k| format!("/files/{k}")),
            created: when(&c.created_at),
        }
    }
}

impl From<&ContactRow> for ContactView {
    fn from(r: &ContactRow) -> Self {
        Self::new(&r.contact, r.company.as_ref())
    }
}

#[derive(Serialize)]
pub struct NoteView {
    pub id: i64,
    pub body: String,
    pub created: String,
    pub contact_id: i64,
    pub contact_name: Option<String>,
}

impl From<&notes::Model> for NoteView {
    fn from(n: &notes::Model) -> Self {
        Self {
            id: n.id,
            body: n.body.clone(),
            created: when(&n.created_at),
            contact_id: n.contact_id,
            contact_name: None,
        }
    }
}

impl From<&RecentNote> for NoteView {
    fn from(r: &RecentNote) -> Self {
        Self {
            contact_name: r.contact.as_ref().map(contacts::Model::full_name),
            ..Self::from(&r.note)
        }
    }
}

#[derive(Serialize)]
pub struct DealView {
    pub id: i64,
    pub title: String,
    pub amount: String,
    pub stage: String,
    pub company: Option<String>,
}

impl DealView {
    #[must_use]
    pub fn new(d: &deals::Model, company: Option<&companies::Model>) -> Self {
        Self {
            id: d.id,
            title: d.title.clone(),
            amount: money(d.amount_cents),
            stage: d.stage.clone(),
            company: company.map(|c| c.name.clone()),
        }
    }
}

#[derive(Serialize)]
pub struct StageView {
    pub stage: String,
    pub count: i64,
    pub total: String,
}

impl From<&StageSummary> for StageView {
    fn from(s: &StageSummary) -> Self {
        Self {
            stage: s.stage.clone(),
            count: s.count,
            total: money(Some(s.total_cents)),
        }
    }
}

#[derive(Serialize)]
pub struct ColumnView {
    pub stage: &'static str,
    pub total: String,
    pub cards: Vec<DealView>,
}

impl From<&BoardColumn> for ColumnView {
    fn from(c: &BoardColumn) -> Self {
        Self {
            stage: c.stage,
            total: money(Some(c.total_cents)),
            cards: c
                .cards
                .iter()
                .map(|card| DealView::new(&card.deal, card.company.as_ref()))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::money;

    #[test]
    fn formats_money() {
        assert_eq!(money(Some(123_456_700)), "$1,234,567");
        assert_eq!(money(Some(99)), "$0");
        assert_eq!(money(None), "$0");
        assert_eq!(money(Some(-500_000)), "-$5,000");
    }
}
