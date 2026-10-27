use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{llm, models::contacts};

const SYSTEM: &str = "You are a CRM assistant. Summarize the relationship with this contact in 3 short \
bullet points: who they are, where the deal stands, and the single best next step. Be concrete.";

pub struct Worker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WorkerArgs {
    pub contact_id: i64,
}

#[async_trait]
impl BackgroundWorker<WorkerArgs> for Worker {
    /// Creates a new instance of the Worker with the given application context.
    ///
    /// This function is called when registering the worker with the queue system.
    ///
    /// # Parameters
    /// * `ctx` - The application context containing shared resources
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    /// Returns the class name of the worker.
    ///
    /// This name is used when enqueueing jobs and identifying the worker in logs.
    /// The implementation returns the struct name as a string.
    fn class_name() -> String {
        "AiSummarize".to_string()
    }

    /// Returns tags associated with this worker.
    ///
    /// Tags can be used to filter which workers run during startup.
    /// The default implementation returns an empty vector (no tags).
    fn tags() -> Vec<String> {
        Vec::new()
    }

    /// Performs the actual work when a job is processed.
    ///
    /// This is the main function that contains the worker's logic.
    /// It gets executed when a job is dequeued from the job queue.
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the job completed successfully, Err otherwise
    async fn perform(&self, args: WorkerArgs) -> Result<()> {
        let contact = contacts::Model::find_by_id(&self.ctx.db, args.contact_id).await?;
        let dossier = contact.dossier(&self.ctx.db).await?;
        // Always write something back: the page polls until the summary is non-null.
        let summary = match llm::complete(&self.ctx, SYSTEM, &dossier).await {
            Ok(c) => c.text,
            Err(e) => {
                tracing::warn!(error = %e, contact_id = args.contact_id, "AI summary failed");
                format!("Summary failed: {e}")
            }
        };
        contact
            .into_active_model()
            .set_ai_summary(&self.ctx.db, Some(summary))
            .await?;
        Ok(())
    }
}
