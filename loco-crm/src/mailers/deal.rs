#![allow(non_upper_case_globals)]

use loco_rs::prelude::*;
use serde_json::json;

use crate::{
    models::{_entities::contacts, deals},
    views::crm::{money, StageView},
};

static shared: Dir<'_> = include_dir!("src/mailers/shared");
static won: Dir<'_> = include_dir!("src/mailers/deal/won");
static digest: Dir<'_> = include_dir!("src/mailers/deal/digest");

#[allow(clippy::module_name_repetitions)]
pub struct Deal {}
impl Mailer for Deal {}
impl Deal {
    /// Tell the sales inbox a deal was won.
    ///
    /// # Errors
    /// When email sending fails.
    pub async fn send_won(
        ctx: &AppContext,
        to: &str,
        deal: &deals::Model,
        contact: Option<&contacts::Model>,
    ) -> Result<()> {
        Self::mail_template_with_shared(
            ctx,
            &won,
            &[&shared],
            mailer::Args {
                to: to.to_string(),
                locals: json!({
                  "title": deal.title,
                  "amount": money(deal.amount_cents),
                  "contact": contact.map(contacts::Model::full_name),
                  "domain": ctx.config.server.full_url()
                }),
                ..Default::default()
            },
        )
        .await
    }

    /// Daily pipeline summary, sent by the `pipeline_digest` task.
    ///
    /// # Errors
    /// When email sending fails.
    pub async fn send_digest(
        ctx: &AppContext,
        to: &str,
        stages: &[deals::StageSummary],
    ) -> Result<()> {
        let open: Vec<_> = stages
            .iter()
            .filter(|s| s.stage != "won" && s.stage != "lost")
            .collect();
        Self::mail_template_with_shared(
            ctx,
            &digest,
            &[&shared],
            mailer::Args {
                to: to.to_string(),
                locals: json!({
                  "stages": stages.iter().map(StageView::from).collect::<Vec<_>>(),
                  "open_count": open.iter().map(|s| s.count).sum::<i64>(),
                  "open_total": money(Some(open.iter().map(|s| s.total_cents).sum())),
                }),
                ..Default::default()
            },
        )
        .await
    }
}
