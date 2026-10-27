//! In-process request profiler.
//!
//! * A middleware times every request, keyed by its matched route, and adds a
//!   `Server-Timing` header (`db`, `app`, `total`) that browser DevTools shows.
//! * SeaORM's metric callback adds each query's duration to a per-request
//!   accumulator (a tokio task-local), so DB time is split out from app time.
//! * Browsers post navigation timings (TTFB, DOMContentLoaded, load) to
//!   `/_profiler/rum`, giving real page-load numbers alongside server ones.
//!
//! Samples live in memory (last `MAX_SAMPLES` per key) and reset on restart.
//! Fine for a POC; production would export to Prometheus/OpenTelemetry.
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        Arc, LazyLock, Mutex,
    },
    time::{Duration, Instant},
};

use axum::{
    extract::{MatchedPath, Request},
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};

const MAX_SAMPLES: usize = 2048;

#[derive(Default)]
struct RouteStats {
    count: u64,
    errors: u64,
    total_ms: VecDeque<f64>,
    db_ms: VecDeque<f64>,
    queries: VecDeque<u32>,
}

#[derive(Default)]
struct RumStats {
    count: u64,
    ttfb: VecDeque<f64>,
    dom_ready: VecDeque<f64>,
    load: VecDeque<f64>,
}

static ROUTES: LazyLock<Mutex<HashMap<String, RouteStats>>> = LazyLock::new(Default::default);
static RUM: LazyLock<Mutex<HashMap<String, RumStats>>> = LazyLock::new(Default::default);
static STARTED: LazyLock<Instant> = LazyLock::new(Instant::now);

#[derive(Default)]
struct RequestDb {
    nanos: AtomicU64,
    queries: AtomicU32,
}

tokio::task_local! {
    static CURRENT: Arc<RequestDb>;
}

fn push<T>(buf: &mut VecDeque<T>, v: T) {
    if buf.len() == MAX_SAMPLES {
        buf.pop_front();
    }
    buf.push_back(v);
}

/// Called from SeaORM's metric callback for every query.
pub fn record_query(elapsed: Duration) {
    let _ = CURRENT.try_with(|acc| {
        let nanos = u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX);
        acc.nanos.fetch_add(nanos, Ordering::Relaxed);
        acc.queries.fetch_add(1, Ordering::Relaxed);
    });
}

/// Axum middleware; installed in `App::after_routes`.
pub async fn middleware(req: Request, next: Next) -> Response {
    LazyLock::force(&STARTED);
    let key = req.extensions().get::<MatchedPath>().map_or_else(
        || format!("{} (static/unmatched)", req.method()),
        |p| format!("{} {}", req.method(), p.as_str()),
    );
    let acc = Arc::new(RequestDb::default());
    let start = Instant::now();
    let mut res = CURRENT.scope(acc.clone(), next.run(req)).await;
    let total = start.elapsed().as_secs_f64() * 1000.0;
    #[allow(clippy::cast_precision_loss)]
    let db = acc.nanos.load(Ordering::Relaxed) as f64 / 1_000_000.0;
    let queries = acc.queries.load(Ordering::Relaxed);

    let timing = format!(
        "db;dur={db:.2};desc=\"{queries} queries\", app;dur={:.2}, total;dur={total:.2}",
        (total - db).max(0.0)
    );
    if let Ok(v) = HeaderValue::from_str(&timing) {
        res.headers_mut().insert("server-timing", v);
    }

    if let Ok(mut routes) = ROUTES.lock() {
        let s = routes.entry(key).or_default();
        s.count += 1;
        if res.status().is_server_error() {
            s.errors += 1;
        }
        push(&mut s.total_ms, total);
        push(&mut s.db_ms, db);
        push(&mut s.queries, queries);
    }
    res
}

#[derive(Debug, Deserialize)]
pub struct RumSample {
    pub path: String,
    pub ttfb: f64,
    pub dom_ready: f64,
    pub load: f64,
}

/// Collapse numeric path segments so `/contacts/17` and `/contacts/42` share a key.
fn normalize_path(path: &str) -> String {
    let path = path.split('?').next().unwrap_or(path);
    path.split('/')
        .map(|seg| {
            if !seg.is_empty() && seg.chars().all(|c| c.is_ascii_digit()) {
                "{id}"
            } else {
                seg
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub fn record_rum(sample: &RumSample) {
    let valid = |v: f64| v.is_finite() && (0.0..120_000.0).contains(&v);
    if !(valid(sample.ttfb) && valid(sample.dom_ready) && valid(sample.load)) {
        return;
    }
    if let Ok(mut rum) = RUM.lock() {
        // Cap distinct keys so arbitrary client paths can't grow memory unbounded.
        let key = normalize_path(&sample.path);
        if !rum.contains_key(&key) && rum.len() >= 200 {
            return;
        }
        let s = rum.entry(key).or_default();
        s.count += 1;
        push(&mut s.ttfb, sample.ttfb);
        push(&mut s.dom_ready, sample.dom_ready);
        push(&mut s.load, sample.load);
    }
}

pub fn reset() {
    if let Ok(mut r) = ROUTES.lock() {
        r.clear();
    }
    if let Ok(mut r) = RUM.lock() {
        r.clear();
    }
}

#[derive(Debug, Serialize, Clone, Copy, Default)]
pub struct Percentiles {
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub max: f64,
}

fn percentiles(values: impl Iterator<Item = f64>) -> Percentiles {
    let mut v: Vec<f64> = values.collect();
    if v.is_empty() {
        return Percentiles::default();
    }
    v.sort_by(f64::total_cmp);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let at = |p: f64| v[(((v.len() - 1) as f64) * p).round() as usize];
    Percentiles {
        p50: at(0.50),
        p95: at(0.95),
        p99: at(0.99),
        max: v[v.len() - 1],
    }
}

#[derive(Debug, Serialize)]
pub struct RouteReport {
    pub route: String,
    pub count: u64,
    pub errors: u64,
    pub total_ms: Percentiles,
    pub db_ms_p50: f64,
    pub queries_p50: f64,
}

#[derive(Debug, Serialize)]
pub struct RumReport {
    pub path: String,
    pub count: u64,
    pub ttfb_ms: Percentiles,
    pub dom_ready_ms: Percentiles,
    pub load_ms: Percentiles,
}

#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub uptime_secs: u64,
    pub rss_mb: Option<f64>,
    pub routes: Vec<RouteReport>,
    pub rum: Vec<RumReport>,
}

#[must_use]
pub fn snapshot() -> Snapshot {
    let mut routes: Vec<RouteReport> = ROUTES
        .lock()
        .map(|r| {
            r.iter()
                .map(|(k, s)| RouteReport {
                    route: k.clone(),
                    count: s.count,
                    errors: s.errors,
                    total_ms: percentiles(s.total_ms.iter().copied()),
                    db_ms_p50: percentiles(s.db_ms.iter().copied()).p50,
                    queries_p50: percentiles(s.queries.iter().map(|q| f64::from(*q))).p50,
                })
                .collect()
        })
        .unwrap_or_default();
    routes.sort_by_key(|r| std::cmp::Reverse(r.count));

    let mut rum: Vec<RumReport> = RUM
        .lock()
        .map(|r| {
            r.iter()
                .map(|(k, s)| RumReport {
                    path: k.clone(),
                    count: s.count,
                    ttfb_ms: percentiles(s.ttfb.iter().copied()),
                    dom_ready_ms: percentiles(s.dom_ready.iter().copied()),
                    load_ms: percentiles(s.load.iter().copied()),
                })
                .collect()
        })
        .unwrap_or_default();
    rum.sort_by_key(|r| std::cmp::Reverse(r.count));

    Snapshot {
        uptime_secs: STARTED.elapsed().as_secs(),
        rss_mb: rss_mb(),
        routes,
        rum,
    }
}

/// Resident set size of this process, via `ps` (works on macOS and Linux).
#[must_use]
pub fn rss_mb() -> Option<f64> {
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    let kb: f64 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
    Some((kb / 1024.0 * 10.0).round() / 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_numeric_segments() {
        assert_eq!(normalize_path("/contacts/42?tab=notes"), "/contacts/{id}");
        assert_eq!(normalize_path("/deals"), "/deals");
    }

    #[test]
    fn percentiles_of_known_values() {
        let p = percentiles((1..=100).map(f64::from));
        assert!((p.p50 - 51.0).abs() < f64::EPSILON || (p.p50 - 50.0).abs() < f64::EPSILON);
        assert!((p.max - 100.0).abs() < f64::EPSILON);
    }
}
