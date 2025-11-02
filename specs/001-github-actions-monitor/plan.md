# Implementation Plan: GitHub Actions Terminal Monitor
**Branch**: `001-github-actions-monitor` | **Date**: 2025-10-27 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-github-actions-monitor/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow. Updated with current status and remaining work based on codebase analysis.

## Summary

Build a Rust-based terminal UI tool that monitors GitHub Actions workflows across multiple repositories using octocrab for API integration and ratatui for the terminal interface. The tool will display the last 4 workflow runs per repository with icon-based status indicators, relative time display, support keyboard navigation, and provide contextual menus for log viewing and browser access. Configuration will be managed through environment variables initially, with layered TOML support planned. Comprehensive validation, error handling, and TDD (with mocks) will ensure robustness. Supports up to 50 repos, <2s refresh, <100MB memory, responsive UI (min 80x24, no overflow).

Current Status: MVP foundational layer implemented (~50% complete). Basic env config, API stubs, simple UI display, and navigation exist. Gaps in full API polling, real-time updates, log fetching, TOML config, and tests. All user stories partially covered; polish needed for production.

## Technical Context

**Language/Version**: Rust 1.75+
**Primary Dependencies**: octocrab=0.38 (API), ratatui=0.29 (UI), tokio=1.0 full (async), serde=1.0 derive (serialization), chrono=0.4 serde (dates), config=0.14 (layered config), crossterm=0.29 (events), env_logger=0.11.8 (logging), clap=4.0 derive (CLI, future), ctrlc=3.4 (signals).
**Storage**: Environment variables (current); TOML files (~/.config/, future); in-memory state (no persistence).
**Testing**: cargo test with mockito for API mocks, proptest for config, tokio-test for async. Current: Basic unit tests pass; comprehensive coverage needed.
**Target Platform**: Linux/macOS/Windows terminals (256-color, Unicode).
**Project Type**: Single binary CLI (standalone, ~6.6MB release).
**Performance Goals**: <2s refresh for 50 repos, <100MB memory, concurrent API with semaphore limits.
**Constraints**: GitHub API rate limits (5k/hr auth), responsive UI no overflow, TDD mandatory.
**Scale/Scope**: Up to 50 repos; last 4 runs/repo; edge cases (rate limits, errors, resize).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design. Updated: Partial compliance – TDD needs tests first for remaining work.*

### TDD Compliance ✅ (Partial)
- All development will follow test-driven development.
- Tests written before implementation.
- Red-Green-Refactor cycle enforced.
- Research phase completed; foundational tests missing – will add mocks before US1-3 extensions.

### Readability and Simplicity ✅
- Single binary with modular architecture (config, github, ui, utils).
- Rust idioms; no unsafe; straightforward patterns (async/await).
- Clear separation: API fetching, state management, rendering.

### Iterative Development ✅
- Broken into phases/user stories (tasks.md); incremental MVP (US1 first).
- Each component testable independently; current MVP runs via env.

### Documentation-First ✅
- Docs in README.md; spec artifacts complete.
- Inline docs partial; add to /docs/ with prefixes post-MVP.
- Latest deps used; config/quickstart.md guides usage.

### External Code Usage ✅
- All from crates.io; no .external mods.
- Dependencies as listed; secrecy added for tokens.

### Rust Quality Standards ✅
- Result-based errors; memory safe.
- Clippy/fmt compliance (run post-implementation).
- Async via Tokio; mocks for tests.

### Testing Requirements ✅ (Pending Full)
- Unit for functions; integration for API/UI.
- Deterministic mocks; clear names.
- Current: Basic; add for coverage (API, config, events).

## Current Implementation Status

Based on codebase analysis against tasks.md:

- **Phase 1 (Setup)**: Completed – Cargo.toml, src/ structure, .env.example, deps.
- **Phase 2 (Foundational)**: 80% – error.rs, config/settings.rs (env), github/client/models (stubs), utils/time/icons, ui/app (state), lib.rs (logging). Gaps: Validation, secrecy, full async.
- **Phase 3 (US1 - Monitoring)**: 60% – Models partial, basic fetching/display (workflow_list/layout), resize no overflow. Gaps: Real-time polling, caching, concurrent fetches. Tests: Missing.
- **Phase 4 (US2 - Navigation)**: 40% – Basic hjkl/Enter, context_menu stub (browser partial, logs placeholder). Tests: Missing.
- **Phase 5 (US3 - Config)**: 30% – Env only; no TOML/CLI/themes/validation. Tests: Missing.
- **Phase 6 (Polish)**: 20% – README good; no full opts, edge tests, inline docs.

Gaps: No tests (critical for TDD); incomplete API/logs; config layering; perf/semaphore. Success criteria partial (US1 partial; US2/US3 gaps).

## Updated Implementation Plan

Follow tasks.md phases iteratively with TDD (tests first, fail, implement, pass, refactor). Prioritize: US1 polish (MVP), then US2/US3 parallel, Polish last. Parallel [P] tasks where file-independent.

### Phases (From tasks.md, with Progress/Remaining)

#### Phase 1: Setup (Completed)
- Project structure, Cargo.toml, .env.example – Done.

#### Phase 2: Foundational (Blocking – 80% → 100%)
- **Remaining**:
  - Add validation (data-model.md rules) to models/config [P].
  - Secure GITHUB_TOKEN with secrecy crate in settings.rs.
  - Full Tokio async in main.rs (polling, signals via ctrlc).
  - Run clippy/fmt; add rustfmt.toml.

#### Phase 3: User Story 1 - Real-time Monitoring (P1 MVP – 60% → 100%)
- **Tests First** (Write & fail):
  - Mock API (mockito) for workflows/runs in tests/unit/github_tests.rs [P].
  - Integration test polling in tests/integration/github_api_tests.rs.
  - UI rendering tests in tests/unit/ui_tests.rs.
- **Implementation**:
  - Full API service (last 4 runs, pagination) in client.rs [P].
  - Auto-refresh (Tokio interval) in ui/app.rs tied to config.
  - Compact/responsive final tweaks (icons/times) in workflow_list/layout.rs.
  - Handle edges: No runs, API errors.

- **Checkpoint**: US1 testable independently (cargo run shows real-time updates, no overflow).

#### Phase 4: User Story 2 - Interactive Navigation & Actions (P1 – 40% → 100%)
- **Tests First** (Write & fail):
  - Mock events for nav/menu in tests/unit/ui_tests.rs [P].
  - Integration menu interactions in tests/integration/ui_tests.rs.
- **Implementation**:
  - Full events (hjkl/Enter/Esc/q/Tab) in ui/app.rs [P].
  - Menu display/actions: Logs fetch (octocrab) & display in context_menu.rs.
  - Browser open (webbrowser crate) for run/repo URLs.
  - PopupState enum in app.rs.

- **Checkpoint**: US1+US2 testable (navigate, view logs, open browser in 3 keys).

#### Phase 5: User Story 3 - Configuration & Customization (P2 – 30% → 100%) [Parallel post-US1]
- **Tests First** (Write & fail):
  - Prop tests (proptest) for parsing in tests/unit/config_tests.rs [P].
  - Integration reload in tests/integration/config_tests.rs.
- **Implementation**:
  - Layered config (env + TOML) in settings.rs via config crate [P].
  - CLI args (--config, --interval, etc.) with clap.
  - Advanced models (Theme/MonitoringConfig) in settings.rs.
  - Validation/reload; theme colors/icons in layout.rs.
  - Update .env.example/quickstart.md.

- **Checkpoint**: All US testable (custom config changes behavior).

#### Phase 6: Polish & Cross-Cutting (Post-US – 20% → 100%)
- Optimize: Semaphore for concurrent API (research.md), backoff/retry (edges).
- Full tests: Cover edges (rates, errors, resize, no-runs); aim 80% coverage.
- Docs: Inline (///), /docs/ folder; update README/quickstart.md (validation flag, systemd).
- Security: Token handling; no logs of secrets.
- Verify: cargo test; manual (50 repos, <2s, SC-001-005); clippy.

**Estimated Timeline**: 3-5 days (TDD adds time); MVP (US1) in 1 day.

## Project Structure

### Documentation (this feature)
```
specs/001-github-actions-monitor/
├── plan.md              # This file
├── research.md          # Phase 0: Tech decisions
├── data-model.md        # Phase 1: Entities/validation
├── quickstart.md        # Phase 1: Usage guide
├── spec.md              # User stories/reqs
├── contracts/           # API schemas (github-api.yaml, config-schema.json)
├── checklists/          # Verification (requirements.md)
└── tasks.md             # Phase 2: Task breakdown
```

### Source Code (repository root)
```
src/
├── main.rs              # Entry: Tokio runtime, UI loop, signals
├── lib.rs               # Pub API, logging init
├── config/
│   ├── mod.rs
│   └── settings.rs      # Layered (env/TOML), validation, models (Theme/Monitoring)
├── github/
│   ├── mod.rs
│   ├── client.rs        # Octocrab wrapper: Fetch runs/repos (semaphore)
│   └── models.rs        # Structs (WorkflowRun, Repository) w/ validation
├── ui/
│   ├── mod.rs
│   ├── app.rs           # State (AppState), events, refresh logic
│   ├── layout.rs        # Responsive layouts (no overflow)
│   └── components/
│       ├── mod.rs
│       ├── workflow_list.rs  # Display runs/icons/times
│       └── context_menu.rs   # Actions (logs, browser)
├── utils/
│   ├── mod.rs
│   ├── time.rs          # Relative formatting (chrono)
│   └── icons.rs         # Status symbols (Unicode)
└── error.rs             # Custom errors (Config/Api/UI)

tests/
├── unit/                # Models/config/utils
│   ├── config_tests.rs
│   ├── github_tests.rs
│   └── ui_tests.rs
└── integration/         # API/UI flows
    ├── github_api_tests.rs
    └── ui_tests.rs

Cargo.toml              # Deps as listed
README.md               # Full usage/architecture
.env.example            # Env template
```
**Structure Decision**: Modular for testing/isolation; follows Rust CLI best practices. Env-first simplifies MVP; TOML for flexibility.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Async Tokio | Real-time polling for 50 repos | Sync blocks UI; rate limits need concurrency |
| Ratatui layouts | Responsive no-overflow UI | Custom rendering complex/error-prone |
| Layered config | User flexibility (env/TOML) | Env-only limits reuse; spec reqs customization |
| Mocks (mockito) | Deterministic tests | Real API rate-limited, non-deterministic |