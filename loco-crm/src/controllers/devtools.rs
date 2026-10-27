//! Developer tools: the in-process request profiler.
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde_json::json;

use crate::{
    controllers::web::CurrentUser,
    profiler::{self, RumSample},
    views::crm::UserView,
};

#[debug_handler]
async fn profiler_page(
    CurrentUser(user): CurrentUser,
    State(_ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    format::render().view(
        &v,
        "devtools/profiler.html",
        json!({ "user": UserView::from(&user), "snap": profiler::snapshot() }),
    )
}

#[debug_handler]
async fn profiler_json(CurrentUser(_user): CurrentUser, State(_ctx): State<AppContext>) -> Result<Response> {
    format::json(profiler::snapshot())
}

#[debug_handler]
async fn profiler_reset(CurrentUser(_user): CurrentUser, State(_ctx): State<AppContext>) -> Result<Response> {
    profiler::reset();
    format::redirect("/_profiler")
}

/// Browser navigation timings, posted by the script in `layouts/base.html`.
#[debug_handler]
async fn rum(Json(sample): Json<RumSample>) -> Result<Response> {
    profiler::record_rum(&sample);
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/_profiler", get(profiler_page))
        .add("/_profiler.json", get(profiler_json))
        .add("/_profiler/reset", post(profiler_reset))
        .add("/_profiler/rum", post(rum))
}
