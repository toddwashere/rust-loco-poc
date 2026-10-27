use loco_rs::prelude::*;

/// `/` goes to the dashboard; `CurrentUser` bounces anonymous visitors to `/login`.
#[debug_handler]
async fn index() -> Result<Response> {
    format::redirect("/dashboard")
}

pub fn routes() -> Routes {
    Routes::new().add("/", get(index))
}
