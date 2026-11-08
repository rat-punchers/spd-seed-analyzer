# AGENTS

Shattered Pixel Dungeon seed analyzer: **Bun + Vite + React + shadcn** UI, **Rust → WASM** engine.

Progress / resume: `specs/implementation.md` (create one if missing and saving progress is requested)
Pinned game: SPD **v4.0.2** @ `57a4e06a4` — local clone often at `/Users/toan/code/repos/00-Evan/shattered-pixel-dungeon`

## Layout

| Path | Role |
|------|------|
| `crates/spd-core` | Pure Rust RNG + generation (no wasm) |
| `crates/spd-objects` | Shared object id enums (items, mobs, rooms, …); source of truth for `web/src/generated/game-ids.ts` |
| `crates/spd-wasm` | `wasm-bindgen` exports |
| `crates/spd-cli` | `spd-analyze` CLI over `spd-core` |
| `web/` | Vite app; `web/src/wasm/` is **generated** |
| `web/public/assets/` | SPD assets (flat; tiles under `environment/`) |
| `brand/design/` | Design system: tokens (`src/theme/theme.css`), restyled shadcn components, spec (`DESIGN.md`), preview app on port 5174 |
| `specs/` | Design / implementation notes |

## Commands

```bash
mise install         # once: rust (rustup-backed), bun, temurin-17, wasm-pack
bun install
bun run dev          # wasm-pack + Vite
bun run build
bun run deploy       # build + Cloudflare Worker SPA (wrangler)
bun run credentials  # guided prod-environment deploy vars + Cloudflare token (--dry-run)
bun run design       # design-system preview (Vite, :5174)
bun run build:design # type-check + build the design-system preview
bun run test:rust    # cargo test -p spd-core
bun run analyze SEED [FLOORS] [--maps] [--pretty] [--full] # exact default-profile JSON (spd-cli)
bun run profile [--seeds N] [--no-wasm] [--open] # engine speed/heap/payload/WASM + hotspots → profile-report/index.html
bun run test:map-render # map-render fixture registry
bun run test:visual  # build + Playwright Chromium pixel comparison
bun run install:visual-browser # once: install Playwright Chromium
bun run build:wasm
bun run gen:ids      # regenerate web/src/generated/game-ids.ts from spd-objects
bun run check        # biome check (TS/JS/CSS/JSON)
bun run check:fix    # biome check --fix
bun run check:rust   # cargo fmt --check + clippy -D warnings (CI)
bun run format       # biome format + cargo fmt
bun run lint         # biome lint + cargo clippy -D warnings
bun run check:all    # biome + rust fmt/clippy checks
```

Local tooling is managed by **mise** (`mise.toml`: rust pinned, rustup-backed).
Keep mise activated in your shell, or prefix commands with `mise exec --`.
mise sets `RUSTUP_TOOLCHAIN`, and the rustup proxies in `~/.cargo/bin` honor
it, so the `PATH` prepends in package.json scripts are no-ops locally (they
exist for CI, which sets up its own toolchain). The
`wasm32-unknown-unknown` target persists in the shared `~/.rustup` store
(`rustup target add wasm32-unknown-unknown` once). Beware: running cargo
outside mise uses the rustup *default* toolchain, which may differ from the
pin (rustfmt output can drift between versions).

### CI parity (before done)

GitHub Actions `check` job (`.github/workflows/ci.yaml`) runs, in order:

1. `bun run check` — Biome  
2. `bun run check:rust` — `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets -- -D warnings`  
3. `bun run test:rust` — `cargo test -p spd-core`  
4. `bun run build` — wasm-pack + Vite production build  
5. `bun run test:visual:only` — Playwright deterministic map comparison

## Rules

- **RNG-PARITY** — Match SPD call order and algorithms (`java.util.Random`, watabou stack, decks). Prefer porting from the pinned clone over inventing shortcuts.
- **DECK-FACTS** — `specs/generator-decks.md` records verified `Generator` deck behaviour for the pinned version: category sub-decks (`seed`/`dropped`), per-depth stream isolation, the full ring draw-site list, which runtime paths bypass the decks, challenge and trinket rule-outs, and measured distributions. Read it before re-deriving any of that from the Java clone or claiming that run history can change a deck-drawn identity. It is settled truth until `PIN-SPD` moves; on a version bump re-verify it and update it in the same change. Add new verified deck findings there — evidence with `file:line` citations and the measurement method, not plans or coverage prose (those belong in `specs/implementation.md`).
- **ANALYSIS-FACTS** — `specs/analysis/` records verified generation facts for the pinned version. Read the relevant analysis before re-deriving behavior from the Java clone or making claims about covered generation paths. Treat it as settled truth until `PIN-SPD` moves; on a version bump re-verify these analyses and update them in the same change.
- **NO-CLAIM** — Do not claim full seed-finder accuracy while status is `partial`. Call out incompleteness.
- **SEED-ANALYSIS** — Focus reports and seed-finder matches on facts determined by the seed for the pinned game version, especially items, loot, and rewards that are guaranteed to spawn. When player state or choices can change the concrete result, analyze the generation path deeply enough to report the possible outcomes and clearly distinguish their conditions while preserving every deterministic property, even when the result is partial. For example, report a guaranteed cursed weapon with `+2` upgrades even when the weapon type varies with player state. Exclude information driven by runtime RNG or otherwise not determined during seed generation, such as combat drops. Keep the public projection consistent across reports, map markers/heaps, WASM, search evidence, and UI.
- **SPAWN-PRESENCE** — For item results, assert only whether the item is guaranteed to spawn. Do not expose internal queue, room-consumption, heap, or placement lifecycle details when they do not change item presence.
- **MAP-LAYOUT-GOAL** — Public floor maps show the painter-complete layout (rooms, terrain, doors, transitions, traps, plants, blobs) plus the generated heaps, mobs, and loot markers, each inspectable on hover/tap. Entities whose identity or placement can change with player state (run history, trinkets) are published in `uncertain_cells` and labelled "May differ"; never let entity uncertainty suppress an otherwise deterministic layout. Areas reachable only through secret doors are published in `hidden_cells` and pulse on the map. Model and label player/meta assumptions that change layout itself.
- **CORE-FIRST** — Generation logic in `spd-core` only; `spd-wasm` stays a thin façade; UI does not reimplement RNG.
- **BUN-WEB** — Package manager is Bun. UI: Vite + React + shadcn. Do not introduce npm/yarn as primary.
- **DEV-SERVER** — You may start or restart the Vite dev server when the web app needs live browser verification. Check whether one is already running first and reuse it. If you started it, stop it when verification is done. Do not leave a server you started running.
- **WASM-REBUILD** — After Rust changes, rebuild wasm (`bun run build:wasm` / `dev`) before treating UI as verified.
- **TEST-RUST** — Add/extend `spd-core` tests for RNG and analyze paths; keep smoke coverage on `analyze_seed`.
- **CI-BEFORE-DONE** — Before marking a task complete, committing work as finished, or claiming “done”, run the same checks as CI’s `check` job (see **CI parity** above). Minimum for any Rust-touching change: `bun run check:rust` and `bun run test:rust`. If TS/web or wasm exports changed, also `bun run check` and `bun run build`. If browser rendering or the visual harness changed, also run `bun run test:visual:only` after the build. If the change affects anything a user sees or interacts with in the web app (UI, layout, styling, routing, or rendered data), also verify in the browser end-to-end: exercise the feature the way a user would, not just a screenshot; check shared routes/state and empty/error variants; check desktop and mobile viewports when layout or styling changed. Start `bun run dev` if needed; if you started it, stop it when done. Fix fmt/clippy/test/build/visual/browser failures before hand-off; do not skip with “clippy later”.
- **DESIGN-SYSTEM** — UI follows `brand/design/DESIGN.md`. Tokens come only from `@spd-seed-analyzer/design/theme.css`; do not add raw palette colors (`text-sky-400`, hex) where a token exists. Shared shadcn components live in `web/src/components/ui/` and are mirrored in `brand/design/src/components/ui/`: change both together. Register any new custom Tailwind scale (`text-*` size, `inset-shadow-*`, `shadow-*`) with tailwind-merge in both `cn()` copies, or `cn()` silently drops or fails to override it. Check the preview (`bun run design`) when tokens or shared components change.
- **ASSETS-FLAT** — Assets live at `web/public/assets/{environment,sprites,…}`. No nested `assets/assets/`.
- **PIN-SPD** — Target the pinned SPD version/commit; note version impact when porting from a newer tree.
- **HAND-OFF** — Optional `specs/implementation.md` is the resume point: current state plus the plan for what comes next. Update it after multi-step work when behavior or next steps change. Keep it **lean and concise** — a short state summary and the ordered next steps, nothing more. It is not a changelog: do not accumulate completed-work history, session logs, or narrative of what was already tried. Rewrite stale sections in place and delete finished ones rather than appending; git history is the record of the past.
- **SMALL-FILES** — Keep source files focused and reviewable. Soft target **≤ ~300 lines**; treat **~500 lines** as a hard ceiling for *new* growth (not an excuse to bloate existing files further). When a change would push a file past ~500, **extract a module first** (same package/`mod`, sibling component, or `lib/` helper) rather than appending. Split by **cohesive responsibility** (room family, UI panel, prize helpers), not arbitrary line cuts. Prefer many small modules + a thin orchestrator over god-files. Does **not** apply to generated output (`web/src/wasm/`), vendored assets, lockfiles, or third-party UI primitives under `web/src/components/ui/` unless we own substantial custom logic there. When expanding an already-oversized file, budget extraction into the same task when practical.
- **CONVENTIONAL-COMMITS** — Commit messages must follow the Conventional Commits format (e.g. `feat: add feature`, `fix: bug fix`, `docs: update docs`). Use types, scopes, bodies, and footers as per the specification for clarity and machine-readability.
