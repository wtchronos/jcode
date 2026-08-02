//! The harness already emits token/cache telemetry. These tests pin the last
//! mile into Desktop2's HUD so the GUI never grows a second accounting system.

use crate::{App, harness};

fn app_with_harness() -> (App, std::sync::mpsc::Sender<harness::HarnessUpdate>) {
    let (update_tx, update_rx) = std::sync::mpsc::channel();
    let (outgoing_tx, _outgoing_rx) = std::sync::mpsc::channel();
    let mut app = App::default();
    app.model.session_id = Some("session_test".into());
    app.model.donut = None;
    app.harness = Some((update_rx, outgoing_tx));
    (app, update_tx)
}

#[test]
fn token_usage_reaches_the_hud_and_turn_done_resets_only_the_snapshot() {
    let (mut app, updates) = app_with_harness();
    updates
        .send(harness::HarnessUpdate::TokenUsage {
            input: 42_800,
            output: 3_100,
            cache_read_input: Some(38_948),
        })
        .expect("queue usage");
    updates
        .send(harness::HarnessUpdate::TurnDone)
        .expect("queue turn end");
    updates
        .send(harness::HarnessUpdate::TokenUsage {
            input: 1_000,
            output: 100,
            cache_read_input: Some(900),
        })
        .expect("queue next-turn usage");

    app.drain_harness_updates();

    assert_eq!(app.model.hud.input_tokens(), 43_800);
    assert_eq!(app.model.hud.output_tokens(), 3_200);
    assert_eq!(app.model.hud.cache_read_tokens(), Some(39_848));
    assert_eq!(app.model.hud.cache_hit_pct(), Some(91));
}
