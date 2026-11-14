# Homelab Dashboard — notes for resuming work

Leptos (CSR/WASM) frontend + Axum backend, single crate with two `[[bin]]`
targets (`homelab-dashboard` = wasm frontend, `server` = Axum). `trunk build`
builds the frontend into `dist/`; the server binary serves `dist/` as a
static fallback alongside its API routes. See `README.md` for the two dev
commands (`trunk build`, `doppler run -- cargo run --bin server`). The
backend must be restarted after backend code changes; the frontend just
needs a `trunk build` (server serves `dist/` live, no restart needed).

Doppler is already configured for this directory (`project homelab`,
`config prd`) — `doppler run -- ...` just works, no extra setup.

## Public SLO status page

Anonymous visitors to `/` now get a public status page instead of the
login screen. Goal: show 28-day SLO + live health for a curated set of
Gatus checks, without authentication, while everything else stays behind
the existing `require_login` middleware untouched.

**Frontend**: `src/frontend/app.rs` renders `PublicStatusPage`
(`src/frontend/pages/public_status.rs`) for `AuthState::Anonymous` instead
of `LoginPage`. This is the only place the login button lived, so hiding
it was just this one swap — `LoginPage` and `/auth/login` still exist and
work, there's just no UI path to them anymore. **That's an open item**:
the user explicitly deferred deciding how to get back to login once the
button is hidden — don't assume a solution, ask.

**Backend routes** (all ungated, added alongside `/auth/*` in
`src/server/mod.rs`, *not* inside the `require_login`-wrapped
`api_routes` sub-router):
- `GET /api/me` — pre-existing handler, just moved out of the gated
  group. Already returned `Json(None)` gracefully for no session.
- `GET /api/public/slo-status` — 28-day headline % + 28-entry daily RAG
  grid per check. Cache refreshes every 5 min (`start_gatus_slo_polling`
  in `src/server/cache.rs`, bespoke loop, not the generic 30s one, since
  a 28-day range query every 30s would be wasteful).
- `GET /api/public/live-status` — Healthy/Unstable/Down per check.
  Cache refreshes every 30s via the generic `start_polling`.

**Which checks get surfaced**: `GATUS_SLO_CHECK_KEYS` in
`src/server/metrics.rs` — a plain `const` list of Gatus `key`s, by
deliberate choice (user wants a hardcoded list, not a Doppler/env var,
since this is expected to keep changing and isn't secret). Currently only
the 9 checks in the "Sites (Allan's Home Lab)" group. More groups/checks
get added here as the user specifies them — check Prometheus
(`https://prometheus.nixos.allanshomelab.com`) label values on `group`/
`key`/`name` for `gatus_results_total` if you need to find more.

**SLO target is three nines (99.9%)**, strict red/green (no amber) on
both the headline pill and the daily boxes — `SLO_TARGET` in
`src/frontend/slo.rs`. Gray boxes mean no data for that day, not a
failure.

**Live state logic** (`fetch_gatus_live_status` in `metrics.rs`): `Down`
if `gatus_results_endpoint_success` is 0 right now; `Unstable` if
currently passing but `homelab_slo_1h < 1.0` (a miss within the last
hour); else `Healthy`. Down always overrides Unstable.

**Headline 28-day number reuses the existing `homelab_slo_28d` recording
rule directly** (instant query) rather than re-deriving the same
`increase()` math ourselves — do the same for any future headline-style
number if an equivalent recording rule already exists (there's also
`homelab_slo_1h` and `homelab_slo_7d` sitting next to it, unused so far
beyond the live-state check).

### PromQL gotcha — read before adding any new regex-filtered query

Gatus check `key`s contain parens (`sites-(allan's-home-lab)_checkmk`).
A regex-escaped key (`sites-\(allan's-home-lab\)_checkmk`) gets rejected
by Prometheus with `unknown escape sequence` — **PromQL string literals
do their own backslash-unescaping before the regex engine ever sees the
pattern**, so a single `\(` is invalid at the query-parsing stage. You
have to double the backslash (`\\(`) so PromQL unescapes it down to the
`\(` the regex engine actually needs. This is handled by
`escape_regex_literal` + `escape_promql_string_literal` (apply both, in
that order) via the `gatus_slo_key_regex()` helper in `metrics.rs` — reuse
that helper, don't hand-roll a new regex-filtered query without it.

### Data depth caveat

Raw `gatus_results_total` counters only go back to ~2026-10-24 (an
apparent Gatus restart reset them); Prometheus retention was separately
bumped from 14 to 35 days around 2026-11-08. The oldest handful of days in
any 28-day window will show `pct: null` (rendered gray) until enough wall
-clock time passes — not a bug, it self-heals.

### Styling (`index.html`)

- `.public-status-content` / `.slo-check-list` / `.slo-check-row` are all
  `width: fit-content` — the whole card shrink-wraps to the width of the
  28-day box grid (the widest thing in each panel) rather than stretching
  full width. If you add something wider than the day-grid to a row,
  check this still looks right.
- `.slo-check-pills .status-pill` overrides the shared `.status-pill`
  padding/font-size/line-height *just inside this page* (fixed width
  `5rem`, `line-height: 1.4rem` matching `.slo-check-name`) so the
  headline and live pills stay aligned and don't resize with their text.
  Don't edit the base `.status-pill` rule for this — other pages
  (hosts/alerts/certs/tasks) use it too.
- Panel background is a single `color-mix(in srgb, var(--button-bg) 92%,
  var(--text) 8%)` shade (the user tried alternating stripes first and
  didn't like it — don't reintroduce striping without asking).
