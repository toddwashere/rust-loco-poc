use fluent_templates::{ArcLoader, FluentLoader};
use loco_rs::controller::views::{engines, ViewRenderer};

/// Tera 2 compiles every template when the engine is built, so building it
/// here catches syntax errors and unknown filters across all of `assets/views`.
/// Renders a view through the same view engine the app builds at
/// boot, including the i18n `t()` function the template calls.
///
/// This covers the whole server-side rendering path end to end: registering a
/// custom function, loading templates that use it, and resolving locales. The
/// template engine validates function references when templates are loaded, so
/// a registration-ordering mistake fails here rather than at runtime.
#[test]
fn every_template_loads_and_login_renders() {
    let loader = std::sync::Arc::new(
        ArcLoader::builder("assets/i18n", unic_langid::langid!("en-US"))
            .shared_resources(Some(&["assets/shared.ftl".into()]))
            .customize(|bundle| bundle.set_use_isolating(false))
            .build()
            .expect("locales should load"),
    );

    let view = engines::TeraView::build_with_post_process(move |tera| {
        tera.register_function("t", FluentLoader::new(loader.clone()));
        Ok(())
    })
    .expect("view engine should build");

    let rendered = view
        .render(
            "auth/login.html",
            serde_json::json!({ "user": null, "error": null, "email": "" }),
        )
        .expect("login view should render");

    assert!(
        rendered.contains("Log in"),
        "expected the login page, got: {rendered}"
    );
}
