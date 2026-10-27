//! `cargo loco task pipeline_digest` — emails the pipeline summary.
//! Scheduled daily in `config/scheduler.yaml`.
use loco_rs::prelude::*;

use crate::{
    mailers::deal::Deal as DealMailer, models::deals, settings::Settings, views::crm::money,
};

pub struct PipelineDigest;
#[async_trait]
impl Task for PipelineDigest {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "pipeline_digest".to_string(),
            detail: "Email the per-stage pipeline summary".to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, _vars: &task::Vars) -> Result<()> {
        let stages = deals::Model::pipeline_summary(&ctx.db).await?;
        for s in &stages {
            println!("{:<10} {:>4} deals  {}", s.stage, s.count, money(Some(s.total_cents)));
        }
        let to = Settings::from_ctx(ctx)?.notify_email;
        DealMailer::send_digest(ctx, &to, &stages).await
    }
}
