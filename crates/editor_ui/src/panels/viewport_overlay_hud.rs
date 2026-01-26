use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::Resource;
use bevy::time::{Real, Time};

#[derive(Resource, Default)]
pub struct ViewportOverlayHudState {
    last_fps_update: f64,
    fps_line: String,
}

const FPS_UPDATE_INTERVAL_SECS: f64 = 0.25;

pub fn update_fps_line<'a>(
    hud: &'a mut ViewportOverlayHudState,
    time: &Time<Real>,
    diagnostics: &DiagnosticsStore,
) -> &'a str {
    let now = time.elapsed_secs_f64();
    if hud.fps_line.is_empty() || now - hud.last_fps_update >= FPS_UPDATE_INTERVAL_SECS {
        hud.last_fps_update = now;
        hud.fps_line = format_fps_line(diagnostics);
    }
    &hud.fps_line
}

fn format_fps_line(diagnostics: &DiagnosticsStore) -> String {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|diagnostic| diagnostic.smoothed());
    let frame_time = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|diagnostic| diagnostic.smoothed());
    match (fps, frame_time) {
        (Some(fps), Some(frame_ms)) => format!("fps={fps:.1} ({frame_ms:.1} ms)"),
        (Some(fps), None) => format!("fps={fps:.1}"),
        _ => "fps=--".to_string(),
    }
}
