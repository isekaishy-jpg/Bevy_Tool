use crate::panels::viewport_overlay_hud::{update_fps_line, ViewportOverlayHudState};
use crate::panels::viewport_overlay_options::{
    draw_overlay_options_window, ViewportOverlayPanelState,
};
use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::{Rect, Vec2};
use bevy::time::{Real, Time};
use bevy_egui::egui;
use editor_core::command_registry::OverlayState;
use editor_core::tools::ActiveTool;
use viewport::{
    subgrid_spacing_meters, SnapKind, ViewportCameraMode, ViewportDebugSettings,
    ViewportInputState, ViewportOverlaySettings, ViewportOverlayStats, ViewportRect,
    ViewportService, ViewportUiInput, ViewportWorldSettings, WorldCursor,
};

pub struct ViewportPanelInputs<'a> {
    pub overlays: &'a OverlayState,
    pub viewport_rect: &'a mut ViewportRect,
    pub viewport_service: &'a mut ViewportService,
    pub viewport_input: &'a ViewportUiInput,
    pub viewport_state: &'a ViewportInputState,
    pub camera_mode: &'a mut ViewportCameraMode,
    pub debug_settings: &'a mut ViewportDebugSettings,
    pub overlay_settings: &'a mut ViewportOverlaySettings,
    pub overlay_stats: &'a ViewportOverlayStats,
    pub world_cursor: &'a WorldCursor,
    pub active_tool: &'a ActiveTool,
    pub world_settings: &'a ViewportWorldSettings,
    pub diagnostics: &'a DiagnosticsStore,
    pub hud_state: &'a mut ViewportOverlayHudState,
    pub time: &'a Time<Real>,
    pub overlay_panel: &'a mut ViewportOverlayPanelState,
}

struct ViewportOverlayUiInputs<'a> {
    viewport_rect: &'a ViewportRect,
    viewport_input: &'a ViewportUiInput,
    viewport_state: &'a ViewportInputState,
    overlay_settings: &'a ViewportOverlaySettings,
    overlay_stats: &'a ViewportOverlayStats,
    world_cursor: &'a WorldCursor,
    active_tool: &'a ActiveTool,
    world_settings: &'a ViewportWorldSettings,
    diagnostics: &'a DiagnosticsStore,
    hud_state: &'a mut ViewportOverlayHudState,
    time: &'a Time<Real>,
}

pub fn draw_viewport_panel(ui: &mut egui::Ui, inputs: &mut ViewportPanelInputs) {
    let panel_rect = ui.max_rect();
    let header_height =
        (ui.spacing().interact_size.y + ui.spacing().item_spacing.y * 2.0).min(panel_rect.height());
    let header_rect = egui::Rect::from_min_size(
        panel_rect.min,
        egui::vec2(panel_rect.width(), header_height),
    );
    let body_rect = egui::Rect::from_min_max(
        egui::pos2(panel_rect.min.x, panel_rect.min.y + header_height),
        panel_rect.max,
    );
    let screen_rect = ui.ctx().input(|input| input.content_rect());
    let scale_factor = ui.ctx().pixels_per_point();
    draw_viewport_header(ui, header_rect, inputs.camera_mode, inputs.overlay_panel);
    update_viewport_rect_from_egui(
        body_rect,
        screen_rect,
        scale_factor,
        inputs.viewport_rect,
        inputs.viewport_service,
    );
    if inputs.overlays.show_overlays {
        let mut overlay_inputs = ViewportOverlayUiInputs {
            viewport_rect: inputs.viewport_rect,
            viewport_input: inputs.viewport_input,
            viewport_state: inputs.viewport_state,
            overlay_settings: inputs.overlay_settings,
            overlay_stats: inputs.overlay_stats,
            world_cursor: inputs.world_cursor,
            active_tool: inputs.active_tool,
            world_settings: inputs.world_settings,
            diagnostics: inputs.diagnostics,
            hud_state: inputs.hud_state,
            time: inputs.time,
        };
        draw_viewport_overlay(ui, body_rect, &mut overlay_inputs);
    }

    draw_overlay_options_window(
        ui.ctx(),
        inputs.overlay_panel,
        inputs.overlay_settings,
        inputs.debug_settings,
    );
}

fn draw_viewport_header(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    camera_mode: &mut ViewportCameraMode,
    overlay_panel: &mut ViewportOverlayPanelState,
) {
    if !egui_rect_is_finite(rect) {
        return;
    }
    let fill = ui.style().visuals.panel_fill;
    ui.painter().rect_filled(rect, 0.0, fill);
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect.shrink(4.0)), |ui| {
        ui.horizontal(|ui| {
            ui.label("Camera");
            ui.separator();
            ui.selectable_value(camera_mode, ViewportCameraMode::Orbit, "Orbit");
            ui.selectable_value(camera_mode, ViewportCameraMode::FreeFly, "Free Fly");
            ui.separator();
            if ui.button("Overlay Options").clicked() {
                overlay_panel.open = !overlay_panel.open;
            }
        });
    });
}

fn draw_viewport_overlay(
    ui: &mut egui::Ui,
    body_rect: egui::Rect,
    inputs: &mut ViewportOverlayUiInputs,
) {
    if !egui_rect_is_finite(body_rect) {
        return;
    }
    let font = egui::FontId::monospace(12.0);
    let color = egui::Color32::from_white_alpha(210);
    let painter = ui.painter();
    let mut hud_y = 6.0;

    if inputs.overlay_settings.show_debug_markers {
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 200, 255));
        painter.rect_stroke(body_rect, 0.0, stroke, egui::StrokeKind::Inside);

        let logical = inputs.viewport_rect.logical_origin;
        let logical_size = inputs.viewport_rect.logical_size;
        let physical = inputs.viewport_rect.physical_origin;
        let physical_size = inputs.viewport_rect.physical_size;
        let rect_line = format!(
            "viewport rect l=({:.1},{:.1}) sz=({:.1},{:.1}) p=({},{}) psz=({},{}) sf={:.2} valid={}",
            logical.x,
            logical.y,
            logical_size.x,
            logical_size.y,
            physical.x,
            physical.y,
            physical_size.x,
            physical_size.y,
            inputs.viewport_rect.scale_factor,
            inputs.viewport_rect.is_valid
        );
        let ui_focus = inputs.viewport_input.wants_pointer || inputs.viewport_input.wants_keyboard;
        let focus_line = format!(
            "focus ui={} hover={} captured={}",
            ui_focus, inputs.viewport_state.hovered, inputs.viewport_state.captured
        );
        let start = body_rect.min + egui::vec2(6.0, 6.0);
        painter.text(
            start,
            egui::Align2::LEFT_TOP,
            rect_line,
            font.clone(),
            color,
        );
        painter.text(
            start + egui::vec2(0.0, 14.0),
            egui::Align2::LEFT_TOP,
            focus_line,
            font.clone(),
            color,
        );
        hud_y += 28.0;
    }

    if inputs.overlay_settings.show_fps {
        let fps_line = update_fps_line(inputs.hud_state, inputs.time, inputs.diagnostics);
        painter.text(
            body_rect.min + egui::vec2(6.0, hud_y),
            egui::Align2::LEFT_TOP,
            fps_line,
            font.clone(),
            color,
        );
        hud_y += 16.0;
    }

    if inputs.overlay_settings.show_cursor_readout && inputs.world_cursor.has_hit {
        let mut lines = Vec::new();
        lines.push(format!(
            "cursor pos=({:.2},{:.2},{:.2})",
            inputs.world_cursor.hit_pos_world.x,
            inputs.world_cursor.hit_pos_world.y,
            inputs.world_cursor.hit_pos_world.z
        ));
        lines.push(format!(
            "region={} tile=({}, {}) chunk=({}, {})",
            inputs.world_cursor.region_id.as_deref().unwrap_or("<none>"),
            inputs.world_cursor.tile_x,
            inputs.world_cursor.tile_y,
            inputs.world_cursor.chunk_x,
            inputs.world_cursor.chunk_y
        ));
        let snap_label = match inputs.world_cursor.snap_kind {
            SnapKind::Off => "off",
            SnapKind::Tile => "tile",
            SnapKind::Chunk => "chunk",
            SnapKind::Subgrid => "subgrid",
        };
        lines.push(format!(
            "tool={} snap={} ({}m)",
            inputs.active_tool.label,
            snap_label,
            subgrid_spacing_meters(inputs.overlay_settings, inputs.world_settings)
        ));
        if let Some(distance) = inputs.world_cursor.edge_distance_tiles {
            if distance < 0 {
                lines.push("bounds=outside region".to_string());
            } else if distance <= 1 {
                lines.push(format!("bounds=near edge ({distance} tiles)"));
            }
        }
        lines.push(format!(
            "overlay lines={} tiles={}",
            inputs.overlay_stats.lines_drawn, inputs.overlay_stats.tiles_considered
        ));

        let start = body_rect.min + egui::vec2(6.0, hud_y.max(36.0));
        for (index, line) in lines.into_iter().enumerate() {
            painter.text(
                start + egui::vec2(0.0, 14.0 * index as f32),
                egui::Align2::LEFT_TOP,
                line,
                font.clone(),
                color,
            );
        }
    }
}

fn update_viewport_rect_from_egui(
    panel_rect: egui::Rect,
    screen_rect: egui::Rect,
    scale_factor: f32,
    viewport_rect: &mut ViewportRect,
    viewport_service: &mut ViewportService,
) {
    if !egui_rect_is_finite(panel_rect)
        || !egui_rect_is_finite(screen_rect)
        || !scale_factor.is_finite()
        || scale_factor <= 0.0
    {
        viewport_rect.invalidate();
        viewport_service.rect = *viewport_rect;
        return;
    }
    let logical = egui_rect_to_bevy(panel_rect);
    let screen = egui_rect_to_bevy(screen_rect);
    let updated = ViewportRect::from_logical_rect(logical, screen, scale_factor);
    *viewport_rect = updated;
    viewport_service.rect = updated;
}

fn egui_rect_to_bevy(rect: egui::Rect) -> Rect {
    Rect::from_corners(
        Vec2::new(rect.min.x, rect.min.y),
        Vec2::new(rect.max.x, rect.max.y),
    )
}

fn egui_rect_is_finite(rect: egui::Rect) -> bool {
    rect.min.x.is_finite()
        && rect.min.y.is_finite()
        && rect.max.x.is_finite()
        && rect.max.y.is_finite()
}
