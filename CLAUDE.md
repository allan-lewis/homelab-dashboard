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

## Shared header/footer across authenticated + public views

The "open item" above is resolved: `AppHeader` (`src/frontend/components/
app_header.rs`) and the new `SiteFooter` (`src/frontend/components/
site_footer.rs`) are now shared by both `AppShell` (authenticated) and
`PublicStatusPage` (anonymous), so the two views stay structurally
aligned instead of diverging.

**`AppHeader`** takes `current_user: Option<User>` — always renders the
header icon + "Allan's Home Lab Dashboard" on the left; right side is
name/email + Logout when `Some`, a Login button (→ `/auth/login`, a
plain `<a>`, no JS) when `None`. This is the way back to `/auth/login`
now that there's no other UI path to it.

**`SiteFooter`** takes `theme_mode`/`set_theme_mode` (always) and an
optional `last_updated: ReadSignal<Option<String>>` (only
`PublicStatusPage` and the authenticated `AppShell` pass one — see
below). Layout: theme picker far left, "© {year} · Allan's Home Lab"
centered (absolute-centered within the footer, independent of how wide
the two side slots are), last-updated far right.

**Menu expand/collapse is gone for now, on purpose** — `menu_state.rs`
(localStorage-backed open/closed flag) was deleted, `SideMenu` no longer
takes a `menu_open` prop, and `.side-menu` is just a fixed 16rem column.
The user explicitly deferred reworking this ("we'll rework that once
everything else is working smoothly") — don't reintroduce a toggle
without asking what shape they want it in.

**Theme picker moved from the sidebar into `SiteFooter`** (it used to
live at the bottom of `SideMenu`, authenticated-only). It's icon-based
now (`ThemeSelector`, inline SVG sun/moon/monitor, `stroke="currentColor"`)
rather than the original "AUTO/LIGHT/DARK" text buttons — deliberately
subtle: no border, `--muted-text` by default, `--hover-bg`/`--active-bg`
only on hover/active. Theme application itself didn't change — `main.rs`
already calls `apply_theme_mode(theme_mode_from_storage())` before mount
regardless of auth state, so system-default + localStorage persistence
was already correct; this only changed where the *control* lives.

**`last_updated` for the authenticated shell is the Overview page's own
signal, lifted up** — `AppShell` now owns `(last_updated,
set_last_updated)` and threads it down through `CurrentPage` →
`OverviewPage` → `OverviewDashboard` (which used to create this signal
itself) so the footer and the Overview "Information" panel
(`OverviewInfoCard`) show the literal same value, not a separately
fetched copy. Only the `Overview` branch of `CurrentPage` takes the
signal — other pages don't have an equivalent "last updated" concept
yet, so the footer's last-updated slot is simply empty on those pages.

### Header icon assets — three separate SVGs, don't conflate them

- `favicon.svg` — browser tab icon. **Hands off this one** unless
  explicitly asked; it's intentionally decoupled from the header icons
  below so they can be iterated on independently.
- `header-icon-dark.svg` / `header-icon-light.svg` — used only inside
  `AppHeader`, swapped via CSS (`.site-header-favicon-dark` /
  `.site-header-favicon-light`, toggled by the same `html[data-theme=...]`
  / `prefers-color-scheme` selector pattern the color vars use at the top
  of `index.html`). Both `<img>` tags are always in the DOM; CSS
  `display` picks which one shows.
- Current design (landed after a few rounds of iteration — see below):
  a house silhouette (`M16 6 L26 15 V26 H6 V15 Z`, white in the dark
  variant / black in the light variant) with its bottom-right 10×10
  quadrant (`x:16–26, y:16–26`) cut out via an evenodd path, and a green
  `rgb(34,197,94)` 7×7 square dropped into that cutout flush to the
  bottom-right corner, inset 3 units from both inner edges — i.e. the
  green square and the house silhouette never touch, with an even
  transparent gap on both sides. No background rect — fully transparent
  surround, blends into whatever's behind the header.
- History worth knowing before re-litigating: earlier attempts used a
  circular badge overlapping the house (looked like a status dot) —
  dropped because the gap between the cutout circle and the badge circle
  was too thin at render size and read as a stray white/black ring even
  after widening it. A green-door-only version (no cutout at all) was
  also tried and reverted — user wanted the quadrant-cutout + inset
  approach specifically. If asked to revisit the icon again, prefer
  small adjustments to the current geometry over reinventing it.
