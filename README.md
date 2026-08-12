# CapDesk — Submission Desk

Desktop app for the auto-submission workflow: **Santi** and **Careem** review a
queue of newly submitted merchant funding deals and decide, one at a time,
whether each deal is **approved** (packet auto-submits to the matched lenders)
or **rejected**. Built in Rust with Tauri; primarily targets Windows.

The UI is a faithful implementation of the **CapDesk** design in
[`design_handoff_capdesk/design_handoff_capdesk/`](design_handoff_capdesk/design_handoff_capdesk/)
(light, flat, green/neutral iOS-style). An earlier handoff spelled the
reviewer "Sanit" — the app uses **Santi** throughout, and the ingest API
accepts both `santiNote` and the old `sanitNote` spelling.

## Architecture

Deals must be receivable **24/7, even when no desk app is open**, and the desks
run on multiple machines. So the system is split:

```
                POST /api/deals  (X-Api-Key: ingest key)
external API ─────────────────────────────► conduit-server  (hosted, always on)
                                             │ axum
                                             ├── Supabase Postgres  (deals, documents)
                                             ├── Supabase Storage   (payload PDFs, seasonality PNGs)
                                             │   auto-submission job on approve
                                             ▼ SSE events  (X-Api-Key: desk key)
                              conduit-desk (Tauri) — Santi's & Careem's machines
                               • commands proxy the HTTP API
                               • SSE relayed as Tauri events → live UI, no polling
                               • desktop notification on every new deal
```

| Piece | Path | Role |
|---|---|---|
| `conduit-core` | `crates/conduit-core` | Shared models + config |
| `conduit-server` | `crates/conduit-server` | Hosted ingest + decision service (axum, sqlx) |
| `conduit-desk` | `src-tauri` + `ui/` | The Tauri desktop app |

**Two API keys, both server-side secrets, both env-configured** (never in the
repo): the *ingest key* lets the external API push deals and nothing else; the
*desk key* is what reviewer apps use for reads, decisions, files, and events.
Approve/reject is atomic in Postgres (`WHERE status = 'awaiting'`), so two
reviewers on different machines can't double-decide a deal.

## Deploying the server

1. Create a Supabase project. From it you need:
   - the Postgres **connection string** (Project Settings → Database — use the
     **session pooler**, port 5432; the transaction pooler breaks prepared
     statements), and
   - the **project URL** + **service_role key** (Project Settings → API) for
     Storage.
2. Generate the two API keys: `openssl rand -hex 32` twice.
3. Copy [`.env.example`](.env.example) and fill it in. Set `TZ` to the
   reviewers' timezone (decision stamps like "Submitted by Santi · 9:31 AM"
   use it).
4. Run the service anywhere that can reach Supabase:
   - **Render** (easiest): push the repo to GitHub, then Render →
     **New → Blueprint** → select the repo. [`render.yaml`](render.yaml)
     defines the service; paste the Supabase values when prompted and Render
     generates the two API keys for you (read them from the service's
     Environment tab). On Render, Supabase Storage is **required** — the
     container disk is wiped on every deploy. Free plan sleeps after ~15 min
     idle (30–60s cold start); upgrade for always-on.
   - **Docker** (any VPS / container platform):
     `docker build -f Dockerfile.server -t conduit-server . && docker run --env-file .env -p 4820:4820 conduit-server`
   - **Bare binary**: `cargo build --release -p conduit-server`, run it with
     the env vars set (systemd unit, NSSM on a Windows box — anything that
     restarts it on failure).
5. Put it behind HTTPS (Caddy/nginx/platform TLS) and give the external API
   the URL + ingest key.

The schema is created automatically on first boot. `SEED_DEMO=true` fills an
*empty* database with the design-handoff demo deals for stakeholder demos.

## Setting up a desk machine (Santi / Careem)

Install the app (see packaging below), then configure where the service lives —
either environment variables `CONDUIT_SERVER_URL` + `CONDUIT_DESK_KEY`, or the
`desk.toml` the app writes on first launch in its data directory
(Windows: `%APPDATA%\CapitalInfusion\Conduit\data\desk.toml`):

```toml
server_url = "https://deals.yourcompany.com"
desk_api_key = "<desk key>"
```

Notifications fire on each desk whenever a new deal arrives (the app can sit
minimized).

**Packaging for Windows:** the
[`Windows desk installer`](.github/workflows/windows-desk.yml) workflow
(Actions tab → Run workflow) builds the `.exe`/`.msi` on a Windows runner.
Pass the server URL and desk key as workflow inputs to bake them in as
defaults — reviewers then just download and run the installer with zero
config (desk.toml / env vars still override). Download from the run's
artifacts, or push a `v*` tag to get a GitHub release with the installers
attached. The installer is unsigned for now, so SmartScreen warns on first
run (More info → Run anyway); rotating the desk key later means rebuilding
baked installers or distributing desk.toml. Only the desk app ships to
reviewers — the server runs in your infrastructure.

### Local development

With no config at all, the desk points at `http://127.0.0.1:4820`, auto-spawns
a local `conduit-server` sitting next to the binary, and borrows the desk key
from the generated `server.toml`. You still need a `DATABASE_URL` (a free
Supabase project works, or any Postgres — e.g.
`docker run -e POSTGRES_PASSWORD=dev -p 5432:5432 postgres:16`), plus
`SEED_DEMO=true` for the demo queue:

```sh
DATABASE_URL=postgresql://postgres:dev@localhost:5432/postgres SEED_DEMO=true cargo run -p conduit-desk
```

Without Supabase Storage credentials, files land on local disk under the data
directory — fine for one box, wrong for production (desks would 404 on PDFs
stored on another machine's disk).

## Ingestion API

```
POST {server}/api/deals
Headers: Content-Type: application/json
         X-Api-Key: <ingest key>
```

Body — the deal record from the source submission sheet (field names per the
design handoff; only `company` and `request` are required):

```json
{
  "company": "Brightline Logistics Corp",
  "email": "ap@brightlinelog.com",
  "phone": "(214) 555-0192",
  "puller": "Marisol Vega",
  "rePuller": "None",
  "state": "TX",
  "tib": "9 yr 4 mo",
  "position": 1,
  "leadSource": "Inbound — Organic search",
  "fico": 702,
  "industry": "Trucking / Freight",
  "revenue": "$148,900 / mo",
  "adb": "$26.8k",
  "deposits": "41 / mo",
  "nsf": 0,
  "request": 150000,
  "lenders": ["East Harbor", "Newtek", "Rapid"],
  "season": [62, 58, 64, 70, 75, 79, 84, 88, 80, 76, 82, 90],
  "seasonalityPng": "<base64 PNG, optional — preferred over season[]>",
  "note": "Clean file.",
  "santiNote": "First position, 700+ FICO — send to the full list.",
  "submittedAt": "2026-08-10T10:33:00-05:00",
  "documents": [
    { "name": "Bank statements — Apr–Jul 2026", "pages": 4, "contentBase64": "<base64 PDF>" }
  ]
}
```

`201` returns the stored deal; connected desks update live and pop a desktop
notification.

Desk-key endpoints: `GET /api/deals[?status=]`, `GET /api/deals/{id}`,
`POST /api/deals/{id}/approve`, `POST /api/deals/{id}/reject` (body
`{"reviewer": "...", "reason": "..."}`), `GET /api/documents/{id}/file`,
`GET /api/deals/{id}/seasonality`, `GET /api/stats`, `GET /api/events` (SSE).
File endpoints also accept `?key=` since images/PDF fetches can't set headers.
`GET /health` is open.

## Design-handoff callouts

Flagged in the handoff as intentionally out of scope or needing business
confirmation:

- **Risk band rule** (`crates/conduit-core/src/risk.rs`) uses the handoff's
  suggested rule — *confirm with the business before shipping* (the handoff's
  own demo data doesn't fully match it).
- **Reviewer toggle is a stand-in for auth**; production should derive the
  reviewer from a signed-in user (Supabase Auth is the natural next step).
- Not yet designed, so not built: bulk approve, J/K + A/R keyboard shortcuts,
  toasts/undo after a decision. (Search *is* implemented, as a
  company/puller/lender filter; reject captures an optional reason inline.)
- **Seasonality** renders the submission sheet's month table (`seasonCalc`)
  or the `season[]` bars. A `seasonalityPng` in the payload is stored by the
  server but not shown on the desk — the sheet data is the source of truth.
- The lender submission job is simulated
  (`crates/conduit-server/src/main.rs`, `approve_deal`) — wire the real
  lender API there.
- Window controls (– ▢ ✕, top right) are an addition the frameless design
  needs; styled in the design language.
- Notifications fire from desks with the app running (including minimized).
  A machine with the app fully closed gets no popup — deals still land and are
  waiting in the queue; if always-on notifications per machine are required,
  the next step is a small tray agent or keeping the app in the tray on close.
