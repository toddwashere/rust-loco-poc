# Loco vs Next.js: hands-on assessment

**Question:** Is Loco a good "batteries-included" default for spinning up startup apps fast, compared with Next.js?

**Short answer:** Not as your default. It's a strong option for a *backend* where infrastructure cost, memory, or correctness matter more than iteration speed.

- At runtime it is far leaner than Next.js:
  - **5x less memory idle.**
  - **~20x less memory under load.**
  - **5–7x the throughput** on framework-bound and server-rendered pages.
- **On a simple SQLite-backed JSON endpoint it only ties.**
- **You pay in iteration speed:** 13–16s rebuilds after every Rust edit, ~1 minute per generator run, and a 90s clean build.
- **You pay in ecosystem:** no official AI SDK, no realtime layer, no admin UI, and OAuth isn't released yet.
- **You pay in project risk:** one main maintainer, and breaking changes inside 1.x.

Online research is in [research.md](research.md). Raw benchmark output is in [../bench/results.txt](../bench/results.txt).

---

## 1. What was built

`loco-crm/` is a small CRM on Loco 1.2.0 with SQLite. `nextjs-twin/` is a minimal Next.js 16 app with the same schema, seed data, and benchmark endpoints.

| Feature | How Loco did it | Next.js equivalent you'd reach for |
|---|---|---|
| Auth (register/login/logout, verify, reset, magic link) | Starter ships it all (JWT). Added an HttpOnly cookie for HTML pages and a `CurrentUser` extractor | Auth.js / Clerk / Better Auth |
| CRUD + relations + migrations | `cargo loco generate scaffold` → migration, SeaORM entity, JSON API, ts-rs DTOs, React pages | Prisma/Drizzle + route handlers + forms |
| Server-rendered UI | **Hand-written** Tera 2 templates + HTMX (live search, inline notes, kanban stage moves) | RSC / server actions |
| React UI | Loco's generated React 19 + TanStack Query SPA, mounted at `/app`, with types generated from Rust | Native |
| Background jobs | SQLite-backed queue: AI summary job, deal-won email job | Inngest / Trigger.dev / QStash (external) |
| Mailer | Tera mail templates with a stub transport in dev/test | Resend / React Email |
| File uploads | `ctx.storage` (disk here, S3 by config change) | UploadThing / S3 SDK |
| Scheduler | Daily `pipeline_digest` task via `scheduler:` config | Vercel Cron / external |
| AI / LLM | Hand-rolled Anthropic Messages API client (reqwest), with a mock fallback; one queued job, one inline call | Vercel AI SDK |
| Profiler | Custom middleware: per-route p50/p95/p99, DB time + query count per request (SeaORM metric hook), `Server-Timing` header, browser page-load beacons, `/_profiler` page | Vercel Analytics / Speed Insights, OTel |
| Tests | 61 tests: model, request (SSR flows with cookie), worker, task, template compile | Vitest + Playwright |

Everything above was exercised in a real browser against the dev server, not only in tests. That covered login, the dashboard, live search, adding a note, a queued AI summary processed by the separate queue worker, an AI draft, an avatar upload, a stage move triggering a queued email job, the React SPA, and the profiler.

---

## 2. Benchmarks

The setup was an Apple M4 Pro (12 cores), both apps in production mode, SQLite with the same 1,000-contact seed, and `oha` for 15s at 50 concurrent connections. RSS is summed over the whole process tree. Endpoints are identical:
- `health`: a JSON `{ok:true}`.
- `api_contacts`: 20 contacts joined to their company, as JSON.
- `ssr_contacts`: the same data rendered as an HTML table.

| | Loco (default pool 10) | Loco (pool 1) | Next.js 16 |
|---|---:|---:|---:|
| Cold start to first 200 | **47 ms** | 99 ms | 546 ms |
| Idle RSS | **24 MB** | 24 MB | 129 MB |
| Peak RSS under load | **33 MB** | 30 MB | **685 MB** |
| `health` req/s (p99) | **46,900** (1.4 ms) | 46,700 | 8,200 (12 ms) |
| `api_contacts` req/s (p99) | 3,280 (17.6 ms) | **4,940** (12.2 ms) | **4,980** (19.2 ms) |
| `ssr_contacts` req/s (p99) | 3,240 (18.5 ms) | **4,430** (14.9 ms) | 616 (101 ms) |

How to read this:

- **Memory is the clearest win.** Loco sat at about 30 MB the whole time. Next.js grew from 129 MB to 685 MB and was still there right after the run ended. On a $5–10 VPS or a small container, that's the difference between one app per box and a dozen. (You wrote "negative memory footprint"; the accurate claim is *tiny and flat*, roughly 20x smaller under load here.)
- **Framework overhead:** Loco serves about 5.7x the requests on the trivial endpoint with a tenth of the tail latency.
- **Server-rendered HTML:** Loco + Tera is 5–7x faster than Next.js SSR, which runs React rendering plus the RSC payload on every request.
- **A DB-bound JSON endpoint is a tie, not a win.** Once SQLite is doing the work, both apps hit about 5k req/s. Next.js's better-sqlite3 is synchronous and in-process, which is very fast for small reads. Loco's stock pool of 10 connections was **slower** (3.3k) than a pool of 1 (4.9k). With SQLite through sqlx, adding connections adds contention: CPU went to 750% while throughput fell. This needs tuning; it isn't free speed.
- **Real apps are usually DB-bound.** For a typical CRUD startup on Postgres, the database caps you long before either framework does. Loco's practical advantage there is **cost per instance** (memory), not raw throughput.

### Does it scale long term?

Yes, in the same way any stateless server scales: JWT auth plus an external queue means you add instances behind a load balancer and run workers as separate processes (`cargo loco start --worker`). Things to know:
- Move off SQLite to Postgres for more than one instance. The Postgres queue backend is supported, so you don't need Redis.
- On Linux, use jemalloc or mimalloc. Loco issue #539 shows RSS ballooning under load with glibc malloc.
- What you give up is Vercel's managed scaling: preview deploys, edge, and zero-ops. You own the container, worker process, migrations, and CI. Compile times also make CI slower: about 90s for a clean release build, versus 6s for `next build`.

---

## 3. What was good (from building it)

1. **The generators are good and do the wiring.** One `scaffold` command produced the migration, entity, JSON API, typed DTOs, TypeScript bindings, TanStack hooks, four React pages, and a test. In Rust, forgetting a `mod` line or route registration is a silent bug, so this matters more than it does in Rails.
2. **The batteries really are integrated.** Jobs, mailer, storage, scheduler, tasks, and tests all hang off one `AppContext`. The deal-won flow chains three of them (request → `DealWon` job → `MailerWorker` job → mail) with no extra infrastructure beyond the SQLite file. In Next.js each of those is a separate vendor decision.
3. **End-to-end types.** Rust DTOs generate TypeScript types (ts-rs), so the React side breaks at compile time when the API changes. That's comparable to tRPC, but across languages.
4. **The test harness is solid.** Request tests boot the real app against a real DB. `ForegroundBlocking` mode runs jobs inline, and the stub mailer captures sent mail. Writing 61 meaningful tests was easy.
5. **The agent skill shipped in the app helped.** The bundled `.claude/skills/loco/` guide (doctrine, API index, recipes) prevented guessed API names and explained the non-obvious traps: `State<AppContext>` being required next to auth extractors, LIFO middleware ordering, and the null cache.
6. **Production config refuses to boot without secrets.** It's a good default: no silent fallback to a dev JWT secret or localhost DB.
7. **Ops footprint:** one ~35 MB binary plus static assets, a 47 ms cold start, and about 25 MB of RAM.

## 4. What was bad (from building it)

Each item below has a commit in this repo that shows the problem and the fix.

1. **The edit-compile loop is slow.** That's 13–16s after a one-line change, 36s to rebuild tests, and ~1 minute per `cargo loco generate` (it recompiles the app to run the generator). Next.js HMR is under a second. Over a day of iteration, this is the single biggest velocity cost.
2. **The HTML/HTMX story regressed in 1.x.** `scaffold --htmx` and `--html` were removed in favour of the React SPA, yet the docs, the screencasts (cast #007), and the bundled agent skill still describe them. So server-rendered CRUD is all hand-written: about 500 lines of controller and 330 lines of templates here.
3. **Documented validation behavior is wrong.** The recipe says a failed `validate()` in `before_save` becomes `ModelError::Validation` (HTTP 400). It actually arrives as a `DbErr` and renders as **500**, and the generated API controllers have this bug out of the box. The fix was to validate explicitly before saving (commit `5c3cad2`). The error's display text is also just "Model validation failed", so you write your own field-message formatting.
4. **The generated React SPA doesn't build** with nullable columns: `Option<T>` → `T | null` → `<input value>` fails type-checking (commit `f14c5fa`). The generated UI is also bare: raw "Company id" number inputs, no pagination controls, no styling.
5. **Several "generate it and it works" gaps:**
   - `generate scheduler` writes a file that `start --all` never reads (commit `e231992`).
   - Tasks don't auto-migrate a fresh DB.
   - The dev DB pool is 1 connection shared with the queue worker. The profiler caught a 46 ms wait on `/api/auth/current` because of it.
   - Generated migrations add no indexes on foreign keys.
   - Mail-subject templates with a trailing newline produce a MIME-encoded subject.
6. **Tera 2 is thin.** It has no date or currency filters, and undefined variables are errors, so every view needs a Rust view-model (`src/views/crm.rs`). Loco moved from Tera 1 to Tera 2 in a *minor* release.
7. **No dev mail outbox.** Stubbed mail is only readable in tests (behind the `testing` feature). In dev you have to run Mailpit, and Docker wasn't running here.
8. **AI integration is DIY.** There's no official Anthropic or OpenAI SDK at the level of the Vercel AI SDK. I wrote about 110 lines of HTTP client, with no streaming, tool use, or UI hooks.
9. **Ecosystem friction compounds.** Stripe, Clerk, PostHog, Resend, Sentry and similar vendors are first-class in Node. In Rust you often call their REST APIs by hand.

---

## 5. Head-to-head summary

| Dimension | Loco | Next.js | Edge |
|---|---|---|---|
| Time to first working CRUD | Fast with generators, then slow compiles | Fast | Next |
| Daily iteration speed | 13–16s per edit | <1s HMR | **Next (big)** |
| Built-in backend batteries (jobs, mail, storage, cron, tasks) | All included and wired | Pick vendors | **Loco** |
| Frontend / UI richness | Tera+HTMX by hand, or generated plain React | Best in class | **Next (big)** |
| Memory / hosting cost | ~25–35 MB | ~130–700 MB | **Loco (big)** |
| Raw throughput (framework-bound, SSR) | 5–7x | baseline | Loco |
| Throughput when DB-bound | ~equal | ~equal | Tie |
| Type safety / refactoring | Compiler-enforced, end-to-end | TypeScript (good) | Loco |
| AI / LLM tooling | DIY | Vercel AI SDK | **Next (big)** |
| Deploy / ops | Own a container + worker + DB | `git push` to Vercel | Next |
| Hiring and AI-assistant fluency | Small pool; agents need the skill file | Huge | **Next** |
| Project risk | ~1 core maintainer, breaking 1.x minors | Vercel-backed | **Next** |
| Mobile backend | Good JSON API (JWT, presigned uploads); no push or realtime | Same, via route handlers | Tie |

## 6. Mobile

Loco can't build mobile apps. It's a backend. As an API for React Native, Expo, or Flutter it's perfectly capable. This POC already serves Bearer-JWT JSON at `/api/*`, the same API the React SPA uses. Missing pieces you'd add: push notifications, websockets (socketioxide), and social login (merged upstream, not released yet).

## 7. Recommendation

- **Keep Next.js as the default** for new startup apps where speed of iteration and UI polish decide whether you win. The edit-compile loop, the frontend gap, and the AI-SDK gap are each enough on their own.
- **Consider Loco when the backend is the product.** That means API-heavy services, job- or worker-heavy systems, or many tenants per box where hosting cost matters. It also fits teams that already know Rust.
- **A hybrid worth prototyping:** a Next.js (or Vite React) frontend with a Loco API and workers behind it. You keep Next's UI and AI-SDK velocity and get Loco's batteries and efficiency on the server. The generated ts-rs types make that boundary typed. The cost is running and deploying two apps.
- **If you adopt it:**
  - Pin the exact `loco-rs` version.
  - Read the release notes before every minor upgrade.
  - Use Postgres and jemalloc in production.
  - Keep the bundled `.claude/skills/loco` up to date for your coding agent.
