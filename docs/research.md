# Loco (loco.rs): desk research

Researched 2026-10-27 from the docs, GitHub, crates.io, the Loco blog, and Hacker News threads. Reddit threads couldn't be fetched, so Reddit sentiment isn't covered. Claims are paraphrased; sources are at the bottom. Hands-on findings from building the POC are in [assessment.md](assessment.md).

## Snapshot

| | |
|---|---|
| What it is | "Rails for Rust": Axum (HTTP) + SeaORM (ORM) + Tokio, with generators and conventions on top |
| Version | **1.2.0** (2026-09-24). 1.0 shipped 2026-07-29 |
| Stars / forks | ~9.5k / ~440, Apache-2.0, created Nov 2023 |
| Downloads | ~722k total, ~139k in the last 90 days |
| Open issues / PRs | 17 / ~11 (604 issues closed) |
| Maintainers | Two people wrote nearly all of the code. In the last 12 months one person wrote ~half of the 95 commits on master, and the co-founder's last commit was Jan 2026 |
| Funding | GitHub Sponsors only. No disclosed company backing |
| Production users | Two self-listed in the README, with no detail on scale. No independent case studies found |

## The batteries

| Battery | What exists | Maturity |
|---|---|---|
| HTTP | Axum handlers and extractors | Mature (it's Axum) |
| ORM / migrations | SeaORM **2.0** (stable since Jul 2026), migration DSL, seeding, pagination | SeaORM 2.0 is months old. Loco's own agent evals found generated SeaORM `Expr` code failing most of the time |
| Auth | JWT (Bearer / cookie / query), API keys, Argon2, a full register/verify/reset/magic-link starter | Good for APIs. OAuth2 (Google/GitHub/OIDC) and server-side sessions were merged 2026-10-19 but **not yet released** |
| Generators | model, migration, scaffold, controller, worker, mailer, task, scheduler, deployment | The core strength. In 1.x, scaffolds emit a React SPA. HTML/HTMX scaffolds were removed (see assessment) |
| Background jobs | Redis / Postgres / SQLite queues, priorities, retry, reaper | Fairly complete. A 1.2 fix covered Redis workers silently stopping after a dropped connection |
| Mailers | Tera templates over SMTP, plus a stub transport for tests | Works |
| Storage | Disk, memory, S3, Azure, GCS. Mirror/backup strategies; presigned URLs since 1.2 | Good |
| Scheduler | Cron or English ("at 8:00 am"); runs tasks as subprocesses | Exists |
| Cache | Null / in-memory / Redis | Basic |
| Testing | `boot_test`, request tests (axum-test), insta snapshots, DB cleanup | Good |
| Deploy | Docker / nginx / AWS Lambda generators (Lambda is HTTP-only, with no workers or scheduler) | Basic |
| Realtime | **None built in.** The docs point to socketioxide | Gap |
| Admin UI | **None** (open feature request) | Gap |
| Agent support | Every new app ships a version-matched `.claude/skills/loco/` skill plus `AGENTS.md` | New in 1.2. Loco's own 10-run eval found the measured gains within noise |

## What people like

- It's the closest thing to Rails ergonomics in Rust. Long-time Rails developers say the conventions map over directly.
- Refactoring confidence: the compiler catches what tests miss. One six-month user said fear of refactoring "is pretty much gone."
- Resource efficiency. The strongest recent report (Oct 2026): a Rails app went from about 700 MB peak RAM to about 60 MB after moving to Loco + HTMX. The author downsized the VPS from 4 GB to 2 GB.
- The maintainers are candid. The agent-eval blog series publishes its own null result.

## What people dislike

- **Compile times** are the most repeated complaint. Feature flags to slim builds landed in Sep–Oct 2026.
- **ORM ergonomics.** Rust ORMs, SeaORM included, still feel verbose next to ActiveRecord. The Loco author has acknowledged the gap.
- **Too much generated code.** One commenter said a scaffold produces about 80 files.
- **Maturity.** Several commenters in 2024–25 called it too early for production.
- **Thin docs and content.** There are 7 short screencasts. The blog was silent from Jan 2024 to Aug 2026, and there is an open issue about doc gaps.
- **Async Rust and Rust itself** are a barrier for some teams.

## Maturity and risk

- **Breaking changes inside 1.x.** 1.1 moved to Tera 2 (templates must be rewritten), changed pagination JSON, and changed auth defaults. 1.2 removed `ExprTrait` from the prelude. No semver policy is published. Before 1.0 there were 32 releases, 11 of them breaking.
- **Bus factor of about 1–2.** There was a 9-month release gap (Oct 2025 to Jul 2026) and master was nearly silent from Mar to Jul 2026.
- **Lock-in is mild.** Underneath, your code is plain Axum + SeaORM + Tokio, so you could keep running it if the project stalled.

## Performance and scaling (published evidence)

- **No Loco-specific benchmarks are published.** Third-party framework benchmarks (TechEmpower and others) put Rust frameworks several times above Node in throughput, but none of those test Loco or Next.js SSR directly.
- **Memory gotcha (issue #539).** With glibc malloc, RSS rose from about 7 MB idle to about 449 MB under heavy load and wasn't returned to the OS. The documented fix is jemalloc or mimalloc. (macOS isn't affected the same way. On Linux, plan to switch allocators.)
- **Scaling model:** stateless app (JWT plus an external queue), horizontal behind a load balancer, workers as separate processes. At startup scale the database is the bottleneck long before Loco is.

## Mobile

Loco works fine as a JSON API for React Native, Expo, or Flutter: Bearer JWT, API keys, and presigned uploads to S3/GCS. Gaps: no push notifications, no realtime/websockets, no Apple Sign-In preset, and OAuth is not yet released. The ts-rs-generated TypeScript types can be shared with a React Native client, but there's nothing equivalent for Flutter.

## AI / LLM in Rust

- **There's no official Anthropic Rust SDK.** Community crates exist (`async-openai`, about 9.9M downloads; `rig-core` for agents/RAG; `genai` for multi-provider), all pre-1.0.
- **Nothing in Rust matches the Vercel AI SDK's UI half** (streaming hooks, generative UI). You hand-roll SSE from Axum, or keep a thin TypeScript layer for AI UI.

## Sources

- https://loco.rs/ · https://loco.rs/casts/ · https://loco.rs/llms.txt · https://loco.rs/blog/
- https://loco.rs/blog/loco-1-1-0/ · https://loco.rs/blog/loco-1-2-0/
- https://loco.rs/blog/agents-1-no-training-data/ · https://loco.rs/blog/agents-4-ten-eval-runs/
- https://loco.rs/docs/how-to/choose-queue-backend/ · https://loco.rs/docs/how-to/deploy/ · https://loco.rs/docs/how-to/build-a-spa/ · https://loco.rs/docs/how-to/websockets/ · https://loco.rs/docs/how-to/configure-storage/
- https://github.com/loco-rs/loco · https://github.com/loco-rs/loco/releases · issues #539, #1674, #1794, #1832, #1842, #1844, #1856, #1859 · PRs #1828, #1876, #1884
- https://crates.io/crates/loco-rs · https://lib.rs/crates/loco-rs
- https://www.sea-ql.org/blog/2026-07-27-sea-orm-2.0/
- Hacker News: items 38540178, 40857308, 41916652, 41917323, 42044957, 42516871, 42605928, 42888832, 44187711, 44189531, 47349934, 50179963, 50184772
- https://github.com/0xPlaygrounds/rig · https://github.com/64bit/async-openai · https://github.com/jeremychone/rust-genai · https://ai-sdk.dev
- https://survey.stackoverflow.co/2025/technology/
