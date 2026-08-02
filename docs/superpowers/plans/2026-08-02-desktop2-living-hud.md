# Desktop2 Living Instrument Panel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a responsive live-run strip and expanded upper-right HUD to Desktop2 using existing harness telemetry and rendering primitives.

**Architecture:** A pure `hud` model accumulates token/cache telemetry and derives display summaries from existing activity, progress, session, model, and failure state. Layout computes optional strip/panel rectangles. A focused renderer draws both surfaces and App pointer routing toggles the panel.

**Tech Stack:** Rust, `jcode-harness-api`, Vello, cosmic-text, existing Desktop2 state-space and pixel capture tests.

## Global Constraints

- No new dependencies.
- Do not fabricate agent or cache data when the API does not report it.
- Keep the HUD inside the conversation viewport.
- Preserve transcript selection, composer, settings, overview, and narrow-window behavior.
- Follow red-green-refactor for every behavioral unit.

---

### Task 1: Pure HUD telemetry state

**Files:**
- Create: `crates/jcode-desktop2/src/hud.rs`
- Modify: `crates/jcode-desktop2/src/main.rs`

**Interfaces:**
- Produces: `hud::Hud`, `Hud::record_usage(input, output, cache_read_input)`, `Hud::cache_hit_pct()`, `Hud::toggle()`, and compact token formatting.

- [ ] Write tests in `hud.rs` proving missing cache telemetry remains unknown, reported turns accumulate without division by zero, cache-hit percentage is bounded, and toggling preserves telemetry.
- [ ] Run `cargo test -p jcode-desktop2 hud::tests -- --nocapture` and confirm the module/API is absent.
- [ ] Implement the smallest pure state model and add `pub hud: hud::Hud` to `Model`.
- [ ] Re-run the focused tests and commit the green unit.

### Task 2: Forward existing API token telemetry

**Files:**
- Modify: `crates/jcode-desktop2/src/harness.rs`
- Modify: `crates/jcode-desktop2/src/app_harness.rs`
- Test: `crates/jcode-desktop2/src/tests/progress.rs` or a new focused `src/tests/hud.rs`

**Interfaces:**
- Produces: `HarnessUpdate::TokenUsage { input, output, cache_read_input }` folded into `Model::hud`.

- [ ] Add a failing harness-update test that sends token usage and expects accumulated HUD totals and cache percentage.
- [ ] Run the single test and confirm it fails because `HarnessUpdate::TokenUsage` does not exist.
- [ ] Map `ApiEvent::TokenUsage` in `harness::run` and fold it in `App::drain_harness_updates`.
- [ ] Re-run the test and commit.

### Task 3: Responsive HUD geometry

**Files:**
- Modify: `crates/jcode-desktop2/src/layout.rs`
- Modify: `crates/jcode-desktop2/src/main.rs`

**Interfaces:**
- Produces: `Frame::run_strip() -> Option<Rect>`, `Frame::hud_panel() -> Option<Rect>`, `Frame::hits_run_strip(x, y)`, and a layout constructor that reserves the strip only when HUD activity exists.

- [ ] Write failing state-space tests: strip never overlaps composer/transcript, panel only appears at the minimum roomy width, panel stays inside the frame, and narrow/degenerate windows remain ordered.
- [ ] Run the layout tests and confirm red.
- [ ] Add HUD constants and responsive geometry while preserving existing constructors for unrelated tests.
- [ ] Pass `model.hud_visible()` from `frame_for_model_with` and re-run the full layout suite.
- [ ] Commit.

### Task 4: Composer run strip rendering and interaction

**Files:**
- Create: `crates/jcode-desktop2/src/scene_hud.rs`
- Modify: `crates/jcode-desktop2/src/scene.rs`
- Modify: `crates/jcode-desktop2/src/main.rs`
- Test: `crates/jcode-desktop2/src/tests/hud.rs`

**Interfaces:**
- Consumes: HUD state and `Frame::run_strip()`.
- Produces: a strip showing live pulse, phase/activity, progress, busy-session dots, token/cache summary, and elapsed time.

- [ ] Add a failing capture/state test asserting semantic capture nodes for phase, activity, progress, and cache summary.
- [ ] Implement strip drawing using existing text, hairline, spinner/progress colors, and the frame clock.
- [ ] Route pointer presses inside the strip to `Hud::toggle()` before composer/transcript handling.
- [ ] Verify clicking outside the strip leaves transcript selection unchanged.
- [ ] Run focused tests and commit.

### Task 5: Expanded Living Instrument Panel

**Files:**
- Modify: `crates/jcode-desktop2/src/scene_hud.rs`
- Modify: `crates/jcode-desktop2/src/scene.rs`
- Test: `crates/jcode-desktop2/src/tests/hud.rs`

**Interfaces:**
- Consumes: `Frame::hud_panel()`, model/provider, activity, progress cards, strip busy sessions, HUD telemetry, memory sample, and failure/recovery state.
- Produces: header, arc gauge, session constellation, telemetry trace, proof/recovery rows, and activity rail.

- [ ] Add failing capture tests for expanded, collapsed, recovery, and narrow-window states.
- [ ] Draw the translucent panel and compact phase arc with existing Vello paths.
- [ ] Add honest conditional rows: omit unknown cache/proof/swarm facts instead of zero-filling.
- [ ] Add sparse pulse/trace animation to `animation_deadline` only while the HUD is live.
- [ ] Run HUD captures and state-space tests, then commit.

### Task 6: Verification, reload, and polish

**Files:**
- Modify only files required by visual fixes.
- Create handoff: `/Users/warren/.hermes/intake/codex/20260802-desktop2-living-hud.md`

- [ ] Run focused tests plus `cargo test -p jcode-desktop2`; record the pre-existing `mem::tests::the_sampler_throttles_between_refreshes` flake separately if it recurs.
- [ ] Run `cargo fmt --check` and the Desktop2 build through `selfdev build-reload target=desktop2`.
- [ ] Use `debug_socket` tester/frame capture to inspect default, active, progress, recovery, collapsed, and narrow states.
- [ ] Compare against the approved mock and make only readability/spacing corrections.
- [ ] Commit the final verified diff.
- [ ] Write the mandatory handoff with provenance, absolute paths, exact test/build output, and known limitations.
