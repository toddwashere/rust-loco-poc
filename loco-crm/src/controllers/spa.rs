//! Hosts the generated React SPA (`frontend/`) under `/app`. Vite builds into
//! `assets/static/app/`, so its JS/CSS are served by the static middleware at
//! `/static/app/*`; every `/app/*` route returns `index.html` for client routing.
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;

const INDEX: &str = "assets/static/app/index.html";

#[debug_handler]
async fn index() -> Result<Response> {
    match tokio::fs::read_to_string(INDEX).await {
        Ok(html) => format::html(&html),
        Err(_) => format::html(
            "<p>The React app is not built yet. Run <code>cd frontend &amp;&amp; npm install &amp;&amp; npm run build</code>.</p>",
        ),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/app", get(index))
        .add("/app/{*rest}", get(index))
}
