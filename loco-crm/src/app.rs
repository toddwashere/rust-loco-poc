use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::storage::{drivers, Storage};
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    bgworker::{BackgroundWorker, Queue},
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    db::{self, truncate_table},
    environment::Environment,
    task::Tasks,
    Result,
};
use migration::Migrator;
use std::{path::Path, sync::Arc};

#[allow(unused_imports)]
use crate::{
    controllers, initializers,
    models::_entities::{companies, contacts, deals, notes, users},
    profiler, tasks,
    workers::downloader::DownloadWorker,
};

pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    async fn after_context(mut ctx: AppContext) -> Result<AppContext> {
        // Uploads go to ./storage on disk; swap for drivers::aws::new(..) in production.
        ctx.storage = Arc::new(Storage::single(drivers::local::new_with_prefix("storage")?));
        // Feed every query's duration into the request profiler.
        ctx.db
            .set_metric_callback(|info| profiler::record_query(info.elapsed));
        Ok(ctx)
    }

    async fn after_routes(router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        Ok(router.layer(axum::middleware::from_fn(profiler::middleware)))
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(
            initializers::view_engine::ViewEngineInitializer,
        )])
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes() // controller routes below
            .add_route(controllers::devtools::routes())
            .add_route(controllers::spa::routes())
            .add_route(controllers::bench::routes())
            .add_route(controllers::web::routes())
            .add_route(controllers::deals::routes())
            .add_route(controllers::contacts::routes())
            .add_route(controllers::companies::routes())
            .add_route(controllers::auth::routes())
            .add_route(controllers::page::routes())
    }
    async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
        queue
            .register(crate::workers::deal_won::Worker::build(ctx))
            .await?;
        queue
            .register(crate::workers::ai_summarize::Worker::build(ctx))
            .await?;
        queue.register(DownloadWorker::build(ctx)).await?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn register_tasks(tasks: &mut Tasks) {
        tasks.register(tasks::seed_demo::SeedDemo);
        tasks.register(tasks::pipeline_digest::PipelineDigest);
        // tasks-inject (do not remove)
        tasks.register(tasks::user_create::UserCreate);
        tasks.register(tasks::user_delete::UserDelete);
    }
    async fn truncate(ctx: &AppContext) -> Result<()> {
        truncate_table(&ctx.db, notes::Entity).await?;
        truncate_table(&ctx.db, deals::Entity).await?;
        truncate_table(&ctx.db, contacts::Entity).await?;
        truncate_table(&ctx.db, companies::Entity).await?;
        truncate_table(&ctx.db, users::Entity).await?;
        Ok(())
    }
    async fn seed(ctx: &AppContext, base: &Path) -> Result<()> {
        db::seed::<users::ActiveModel>(&ctx.db, &base.join("users.yaml").display().to_string())
            .await?;
        Ok(())
    }
}
