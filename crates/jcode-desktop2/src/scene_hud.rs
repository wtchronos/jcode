use crate::text::ParagraphStyle;
use crate::{Model, hud, layout, text};
use vello::Scene;
use vello::kurbo::{Affine, BezPath, Circle, Rect, RoundedRect};
use vello::peniko::{Color, Fill};

pub fn draw_hud(
    scene: &mut Scene,
    text: &mut text::TextSystem,
    model: &Model,
    frame: &layout::Frame,
    scale: f64,
    now: std::time::Instant,
) {
    if let Some(strip) = frame.run_strip() {
        draw_run_strip(scene, text, model, strip, scale, now);
    }
    if let Some(panel) = frame.hud_panel() {
        draw_panel(scene, text, model, panel, scale, now);
    }
}

fn draw_run_strip(
    scene: &mut Scene,
    text: &mut text::TextSystem,
    model: &Model,
    strip: Rect,
    scale: f64,
    now: std::time::Instant,
) {
    let theme = &model.theme;
    scene.fill(
        Fill::NonZero,
        Affine::scale(scale),
        theme.wash,
        None,
        &RoundedRect::from_rect(strip, layout::COMPOSER_RADIUS),
    );
    scene.stroke(
        &vello::kurbo::Stroke::new(1.0 / scale),
        Affine::scale(scale),
        theme.rule,
        None,
        &RoundedRect::from_rect(strip, layout::COMPOSER_RADIUS),
    );

    let pulse = Circle::new((strip.x0 + layout::HUD_PAD, strip.center().y), 3.0);
    scene.fill(
        Fill::NonZero,
        Affine::scale(scale),
        if model.busy { theme.text } else { theme.muted },
        None,
        &pulse,
    );

    let phase = if model.busy {
        model.activity.label().to_string()
    } else if model.failure.is_some() {
        "recovery".into()
    } else {
        "ready".into()
    };
    let progress = progress_summary(model);
    let cache = cache_summary(model);
    let elapsed = model
        .activity
        .line(now)
        .and_then(|line| line.split('·').nth(1).map(str::trim).map(str::to_string))
        .unwrap_or_else(|| "idle".into());
    let line = format!(
        "{} · {} · {} · {} · {}",
        phase,
        model.activity.label(),
        progress,
        cache,
        elapsed
    );
    text.draw_paragraph_scaled(
        scene,
        &crate::scene::elide(&line, ((strip.width() - 36.0) / 6.4).max(12.0) as usize),
        (strip.x0 + 24.0, strip.y0 + 3.0),
        (strip.width() - 32.0).max(1.0) as f32,
        ParagraphStyle {
            font_size: layout::CAPTION_SIZE,
            color: theme.muted,
            letter_spacing_em: 0.08,
            ..Default::default()
        },
        scale,
    );
}

fn draw_panel(
    scene: &mut Scene,
    text: &mut text::TextSystem,
    model: &Model,
    panel: Rect,
    scale: f64,
    now: std::time::Instant,
) {
    let theme = &model.theme;
    scene.fill(
        Fill::NonZero,
        Affine::scale(scale),
        theme.field.with_alpha(0.94),
        None,
        &RoundedRect::from_rect(panel, layout::HUD_PANEL_RADIUS),
    );
    scene.stroke(
        &vello::kurbo::Stroke::new(1.0 / scale),
        Affine::scale(scale),
        theme.rule,
        None,
        &RoundedRect::from_rect(panel, layout::HUD_PANEL_RADIUS),
    );

    let x = panel.x0 + layout::HUD_PAD;
    let mut y = panel.y0 + layout::HUD_PAD;
    label(
        text,
        scene,
        "Living Instrument Panel",
        x,
        y,
        panel.width(),
        theme.text,
        scale,
    );
    y += 24.0;

    draw_arc(
        scene,
        panel,
        theme.muted,
        scale,
        model.busy || model.failure.is_some(),
    );
    row(
        text,
        scene,
        "phase",
        phase(model),
        x,
        y,
        panel.width(),
        theme,
        scale,
    );
    y += 20.0;
    row(
        text,
        scene,
        "activity",
        model.activity.label(),
        x,
        y,
        panel.width(),
        theme,
        scale,
    );
    y += 20.0;
    row(
        text,
        scene,
        "progress",
        &progress_summary(model),
        x,
        y,
        panel.width(),
        theme,
        scale,
    );
    y += 20.0;
    row(
        text,
        scene,
        "tokens",
        &token_summary(model),
        x,
        y,
        panel.width(),
        theme,
        scale,
    );
    y += 20.0;
    row(
        text,
        scene,
        "cache",
        &cache_summary(model),
        x,
        y,
        panel.width(),
        theme,
        scale,
    );
    y += 20.0;
    let elapsed = model.activity.elapsed(now).as_secs();
    row(
        text,
        scene,
        "elapsed",
        &format_elapsed(elapsed),
        x,
        y,
        panel.width(),
        theme,
        scale,
    );
    y += 20.0;
    if let Some(proof) = model.hud.latest_proof() {
        row(
            text,
            scene,
            "proof",
            proof,
            x,
            y,
            panel.width(),
            theme,
            scale,
        );
        y += 20.0;
    }
    if let Some(failure) = model.failure.as_deref() {
        row(
            text,
            scene,
            "recovery",
            failure,
            x,
            y,
            panel.width(),
            theme,
            scale,
        );
    }
}

fn draw_arc(scene: &mut Scene, panel: Rect, color: Color, scale: f64, hot: bool) {
    let cx = panel.x1 - 35.0;
    let cy = panel.y0 + 35.0;
    let mut path = BezPath::new();
    path.move_to((cx, cy - 18.0));
    path.quad_to((cx + 20.0, cy - 18.0), (cx + 18.0, cy + 5.0));
    path.quad_to((cx + 16.0, cy + 20.0), (cx - 6.0, cy + 18.0));
    scene.stroke(
        &vello::kurbo::Stroke::new(if hot { 2.0 } else { 1.0 }),
        Affine::scale(scale),
        color,
        None,
        &path,
    );
}

fn label(
    text: &mut text::TextSystem,
    scene: &mut Scene,
    value: &str,
    x: f64,
    y: f64,
    width: f64,
    color: Color,
    scale: f64,
) {
    text.draw_paragraph_scaled(
        scene,
        value,
        (x, y),
        width as f32,
        ParagraphStyle {
            font_size: layout::BODY_SIZE,
            color,
            ..Default::default()
        },
        scale,
    );
}

fn row(
    text: &mut text::TextSystem,
    scene: &mut Scene,
    key: &str,
    value: &str,
    x: f64,
    y: f64,
    width: f64,
    theme: &crate::theme::Theme,
    scale: f64,
) {
    let line = format!("{key}: {}", crate::scene::elide(value, 38));
    text.draw_paragraph_scaled(
        scene,
        &line,
        (x, y),
        (width - layout::HUD_PAD * 2.0).max(1.0) as f32,
        ParagraphStyle {
            font_size: layout::CAPTION_SIZE,
            color: theme.muted,
            letter_spacing_em: 0.06,
            ..Default::default()
        },
        scale,
    );
}

fn phase(model: &Model) -> &'static str {
    if model.failure.is_some() {
        "recovery"
    } else if model.busy {
        "running"
    } else {
        "ready"
    }
}

fn progress_summary(model: &Model) -> String {
    model
        .transcript
        .messages()
        .iter()
        .rev()
        .find(|message| message.role == crate::transcript::Role::Progress)
        .map(|message| message.source.clone())
        .unwrap_or_else(|| {
            if model.busy {
                "live".into()
            } else {
                "idle".into()
            }
        })
}

fn token_summary(model: &Model) -> String {
    format!(
        "{} in / {} out",
        hud::compact_tokens(model.hud.input_tokens()),
        hud::compact_tokens(model.hud.output_tokens())
    )
}

fn cache_summary(model: &Model) -> String {
    model
        .hud
        .cache_hit_pct()
        .map(|pct| format!("{pct}% hit"))
        .unwrap_or_else(|| "cache unknown".into())
}

fn format_elapsed(seconds: u64) -> String {
    if seconds < 60 {
        format!("{seconds}s")
    } else {
        format!("{}m{:02}s", seconds / 60, seconds % 60)
    }
}
