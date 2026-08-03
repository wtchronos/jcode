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

    let phase = phase(model).to_uppercase();
    let progress = progress_pct(model)
        .map(|pct| format!("{pct}%"))
        .unwrap_or_else(|| "LIVE".into());
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
        token_summary(model),
        format!("KV {cache} · {elapsed}")
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
    let inner_width = panel.width() - layout::HUD_PAD * 2.0;
    let header = model
        .model
        .as_ref()
        .and_then(crate::ModelId::caption)
        .unwrap_or_else(|| "Jcode".into());
    label(
        text,
        scene,
        &header,
        x,
        panel.y0 + 10.0,
        inner_width,
        theme.text,
        scale,
    );
    caption(
        text,
        scene,
        &phase(model).to_uppercase(),
        panel.x1 - 72.0,
        panel.y0 + 12.0,
        60.0,
        if model.failure.is_some() {
            theme.removed
        } else if model.busy {
            theme.added
        } else {
            theme.muted
        },
        scale,
    );

    let rule = BezPath::from_vec(vec![
        vello::kurbo::PathEl::MoveTo((x, panel.y0 + 34.0).into()),
        vello::kurbo::PathEl::LineTo((panel.x1 - layout::HUD_PAD, panel.y0 + 34.0).into()),
    ]);
    scene.stroke(
        &vello::kurbo::Stroke::new(1.0 / scale),
        Affine::scale(scale),
        theme.rule,
        None,
        &rule,
    );

    let gauge_center = (x + 38.0, panel.y0 + 78.0);
    draw_gauge(
        scene,
        gauge_center,
        theme.rule,
        if model.failure.is_some() {
            theme.removed
        } else {
            theme.added
        },
        scale,
        progress_pct(model).unwrap_or(if model.busy { 18 } else { 100 }),
    );
    label(
        text,
        scene,
        &progress_pct(model)
            .map(|pct| format!("{pct}%"))
            .unwrap_or_else(|| "LIVE".into()),
        gauge_center.0 - 21.0,
        gauge_center.1 - 10.0,
        44.0,
        theme.text,
        scale,
    );
    caption(
        text,
        scene,
        phase(model),
        gauge_center.0 - 21.0,
        gauge_center.1 + 9.0,
        44.0,
        theme.faint,
        scale,
    );

    let detail_x = x + 88.0;
    caption(
        text,
        scene,
        "CURRENT ACTIVITY",
        detail_x,
        panel.y0 + 48.0,
        inner_width - 88.0,
        theme.faint,
        scale,
    );
    label(
        text,
        scene,
        &crate::scene::elide(model.activity.label(), 22),
        detail_x,
        panel.y0 + 63.0,
        inner_width - 88.0,
        theme.text,
        scale,
    );
    let elapsed = model.activity.elapsed(now).as_secs();
    caption(
        text,
        scene,
        &format!(
            "{} · {}",
            format_elapsed(elapsed),
            crate::scene::elide(&progress_summary(model), 22)
        ),
        detail_x,
        panel.y0 + 84.0,
        inner_width - 88.0,
        theme.muted,
        scale,
    );

    let telemetry_y = panel.y0 + 119.0;
    telemetry(
        text,
        scene,
        "INPUT",
        &hud::compact_tokens(model.hud.input_tokens()),
        x,
        telemetry_y,
        62.0,
        theme,
        scale,
    );
    telemetry(
        text,
        scene,
        "OUTPUT",
        &hud::compact_tokens(model.hud.output_tokens()),
        x + 68.0,
        telemetry_y,
        62.0,
        theme,
        scale,
    );
    telemetry(
        text,
        scene,
        "KV CACHE",
        &cache_summary(model),
        x + 136.0,
        telemetry_y,
        72.0,
        theme,
        scale,
    );
    telemetry(
        text,
        scene,
        "MEMORY",
        &model
            .mem
            .as_ref()
            .map(crate::mem::Readout::caption)
            .unwrap_or_else(|| "--".into()),
        x + 214.0,
        telemetry_y,
        inner_width - 214.0,
        theme,
        scale,
    );

    let event_y = panel.y0 + 158.0;
    let (event_kind, event, event_color) = if let Some(failure) = model.failure.as_deref() {
        ("RECOVERY", failure, theme.removed)
    } else if let Some(proof) = model.hud.latest_proof() {
        ("LATEST PROOF", proof, theme.added)
    } else {
        ("LIVE SIGNAL", "waiting for proof", theme.muted)
    };
    caption(
        text,
        scene,
        event_kind,
        x,
        event_y,
        86.0,
        event_color,
        scale,
    );
    caption(
        text,
        scene,
        &crate::scene::elide(event, 24),
        x + 90.0,
        event_y,
        inner_width - 90.0,
        theme.muted,
        scale,
    );

    draw_session_map(
        scene,
        text,
        model,
        Rect::new(
            x,
            panel.y0 + 184.0,
            panel.x1 - layout::HUD_PAD,
            panel.y0 + 244.0,
        ),
        scale,
    );
}

fn draw_session_map(
    scene: &mut Scene,
    text: &mut text::TextSystem,
    model: &Model,
    bounds: Rect,
    scale: f64,
) {
    let theme = &model.theme;
    let topology = hud::SessionTopology::from_strip(&model.strip);
    let summary = format!(
        "{} sessions · {} live · {} groups",
        topology.sessions, topology.busy, topology.groups
    );
    caption(
        text,
        scene,
        "SESSION MAP",
        bounds.x0,
        bounds.y0,
        92.0,
        theme.faint,
        scale,
    );
    caption(
        text,
        scene,
        &summary,
        bounds.x0 + 96.0,
        bounds.y0,
        bounds.width() - 96.0,
        theme.muted,
        scale,
    );

    let entries = model.strip.entries();
    if entries.is_empty() {
        caption(
            text,
            scene,
            "waiting for daemon session facts",
            bounds.x0,
            bounds.y0 + 30.0,
            bounds.width(),
            theme.muted,
            scale,
        );
        return;
    }

    let center = (bounds.x0 + bounds.width() * 0.52, bounds.y0 + 41.0);
    let radius = (bounds.width() * 0.31).min(86.0);
    let focused = model.strip.focused_session();
    let visible = entries.len().min(8);
    let focused_entry = entries
        .iter()
        .find(|entry| focused == Some(entry.session_id.as_str()));
    let orbit: Vec<_> = entries
        .iter()
        .filter(|entry| focused != Some(entry.session_id.as_str()))
        .take(visible.saturating_sub(1))
        .collect();
    let mut points = Vec::with_capacity(visible);
    if let Some(entry) = focused_entry {
        points.push((center, entry));
    }
    for (index, entry) in orbit.iter().enumerate() {
        let angle = -std::f64::consts::FRAC_PI_2
            + std::f64::consts::TAU * index as f64 / orbit.len().max(1) as f64;
        let point = (
            center.0 + angle.cos() * radius,
            center.1 + angle.sin() * 17.0,
        );
        points.push((point, *entry));
    }

    for (point, entry) in &points {
        if focused == Some(entry.session_id.as_str()) {
            continue;
        }
        let mut link = BezPath::new();
        link.move_to(center);
        link.line_to(*point);
        scene.stroke(
            &vello::kurbo::Stroke::new(1.0 / scale),
            Affine::scale(scale),
            theme.rule,
            None,
            &link,
        );
    }

    for (point, entry) in points {
        let is_focused = focused == Some(entry.session_id.as_str());
        let node_radius = if is_focused {
            7.0
        } else if entry.busy {
            5.0
        } else {
            4.0
        };
        if is_focused {
            scene.stroke(
                &vello::kurbo::Stroke::new(1.5 / scale),
                Affine::scale(scale),
                theme.added.with_alpha(0.45),
                None,
                &Circle::new(point, node_radius + 4.0),
            );
        }
        scene.fill(
            Fill::NonZero,
            Affine::scale(scale),
            if entry.busy { theme.added } else { theme.muted },
            None,
            &Circle::new(point, node_radius),
        );
    }

    if entries.len() > visible {
        caption(
            text,
            scene,
            &format!("+{}", entries.len() - visible),
            bounds.x1 - 24.0,
            bounds.y0 + 48.0,
            24.0,
            theme.muted,
            scale,
        );
    }
}

fn draw_gauge(
    scene: &mut Scene,
    center: (f64, f64),
    track: Color,
    signal: Color,
    scale: f64,
    progress: u8,
) {
    scene.stroke(
        &vello::kurbo::Stroke::new(2.0),
        Affine::scale(scale),
        track,
        None,
        &Circle::new(center, 30.0),
    );
    let sweep = f64::from(progress.min(100)) / 100.0;
    let cx = center.0;
    let cy = center.1;
    let end = std::f64::consts::TAU * sweep - std::f64::consts::FRAC_PI_2;
    let mut path = BezPath::new();
    let steps = 32;
    for step in 0..=steps {
        let angle = -std::f64::consts::FRAC_PI_2
            + (end + std::f64::consts::FRAC_PI_2) * f64::from(step) / f64::from(steps);
        let point = (cx + angle.cos() * 30.0, cy + angle.sin() * 30.0);
        if step == 0 {
            path.move_to(point);
        } else {
            path.line_to(point);
        }
    }
    scene.stroke(
        &vello::kurbo::Stroke::new(3.0),
        Affine::scale(scale),
        signal,
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

fn caption(
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
        width.max(1.0) as f32,
        ParagraphStyle {
            font_size: layout::CAPTION_SIZE,
            color,
            letter_spacing_em: 0.06,
            ..Default::default()
        },
        scale,
    );
}

#[allow(clippy::too_many_arguments)]
fn telemetry(
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
    caption(text, scene, key, x, y, width, theme.faint, scale);
    caption(
        text,
        scene,
        &crate::scene::elide(value, (width / 6.5).max(4.0) as usize),
        x,
        y + 17.0,
        width,
        theme.text,
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

fn progress_pct(model: &Model) -> Option<u8> {
    let source = progress_summary(model);
    source
        .split_whitespace()
        .find_map(|part| {
            part.trim_matches(|ch: char| !ch.is_ascii_digit() && ch != '%')
                .strip_suffix('%')
        })
        .and_then(|value| value.parse::<u8>().ok())
        .filter(|value| *value <= 100)
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
