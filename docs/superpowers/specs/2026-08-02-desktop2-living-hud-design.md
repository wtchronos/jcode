# Desktop2 Living Instrument Panel Design

**Approved visual target:** `/Users/warren/.jcode/source/jcode/.jcode/generated-images/1785712645064-ig_09da451d6b6d13b9016a6fcfbd04748196a3a8f6c4245a288f.png`

## Goal

Keep long Jcode runs visible, pleasant, and useful without leaving the conversation. A slim run strip sits immediately above the composer. On roomy windows, a docked/floating instrument panel uses the upper-right negative space. Both surfaces are views over the same live HUD state.

## Visual language

- Pure-black Desktop2 canvas with hairline geometry and restrained translucent surfaces.
- Green means live or verified, blue means active tools/sessions, amber means recovery.
- Motion is informative and sparse: breathing live pulse, progress sweep, telemetry trace, fresh-proof shimmer.
- Compact, technical, and native to Desktop2. No detached dashboard, side navigation, or generic analytics tiles.

## Information hierarchy

### Persistent run strip

Visible while a turn, tool, or background task is active. It shows phase, progress, busy-session dots, current activity, token/cache summary, and elapsed time. Clicking the strip toggles the expanded panel.

### Expanded panel

Visible by default on windows wide enough to hold it without obscuring the readable text measure. It shows:

1. Provider/model and reasoning level when available.
2. A compact progress ring or arc with current phase and activity.
3. Busy session nodes derived from the existing session strip.
4. Token input/output, KV cache hit rate, process memory, and small telemetry traces.
5. Latest verified event and current recovery/failure signal.
6. A short recent-activity rail derived from existing tool, progress, and turn events.

On narrower windows, only the run strip remains. The panel can always be collapsed.

## Architecture

- `hud.rs` owns pure HUD state and presentation-ready summaries. It consumes existing model/activity/progress/session data plus `ApiEvent::TokenUsage`.
- `layout.rs` computes responsive run-strip and panel rectangles. It reserves vertical room for the run strip and suppresses the panel below a minimum width.
- `scene_hud.rs` renders the strip and panel with existing Vello/text primitives.
- `harness.rs` forwards token telemetry already emitted by `jcode-harness-api`.
- `app_harness.rs` folds telemetry into HUD state.
- `main.rs` stores HUD state and routes pointer toggles.

No new dependency or telemetry protocol is required.

## Interaction and fallback

- Clicking the run strip toggles expanded/collapsed state.
- The panel never intercepts transcript selection outside its own bounds.
- Missing telemetry removes that datum instead of showing a fabricated zero.
- Agent count is not guessed. The first version labels existing daemon facts as active sessions unless a typed swarm event is available.
- Failures/recovery remain visible in the conversation and footnote. The HUD mirrors them but is never the sole error surface.

## Verification

- Unit tests for token/cache accumulation and summary formatting.
- Layout state-space tests for no overlap, responsive panel suppression, and composer visibility.
- Harness event tests proving `TokenUsage` reaches HUD state.
- Scene capture nodes for collapsed, running, progress, cache, recovery, and narrow-window states.
- Desktop2 build/reload and visual comparison against the approved mock.
