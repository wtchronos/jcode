//! The harness already emits token/cache telemetry. These tests pin the last
//! mile into Desktop2's HUD so the GUI never grows a second accounting system.

use crate::{App, harness};
use vello::Scene;

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

#[test]
fn pressing_the_run_strip_toggles_the_expanded_hud() {
    let mut app = App::default();
    app.model.donut = None;
    app.model.working_dir = Some("/home/j/jcode".into());
    app.model.hud.record_usage(42_800, 3_100, Some(38_948));
    app.frame = App::frame_for_model((1280, 760), 1.0, &app.model);
    let strip = app.frame.run_strip().expect("run strip should be visible");
    assert!(app.model.hud.expanded());

    app.pointer = (strip.center().x, strip.center().y);
    app.on_pointer_pressed();

    assert!(!app.model.hud.expanded());
}

#[test]
fn clicking_outside_the_run_strip_does_not_toggle_hud() {
    let mut app = App::default();
    app.model.donut = None;
    app.model.working_dir = Some("/home/j/jcode".into());
    app.model.hud.record_usage(1_000, 100, Some(900));
    app.frame = App::frame_for_model((1280, 760), 1.0, &app.model);

    app.pointer = (app.frame.left + 4.0, app.frame.body_top + 4.0);
    app.on_pointer_pressed();

    assert!(app.model.hud.expanded());
    assert!(app.model.selection.is_none());
}

#[test]
fn living_hud_capture_nodes_render_collapsed_expanded_recovery_and_narrow_states() {
    for node in [
        "living_hud_expanded",
        "living_hud_collapsed",
        "living_hud_recovery",
        "living_hud_narrow",
    ] {
        let model = crate::states::by_name(node).expect("state-space node exists");
        let mut scene = Scene::new();
        let mut painter = crate::paint::Painter::default();
        crate::scene::build_scene(&mut scene, &mut painter, &model, (1280, 760), 1.0);
    }
}
