use bevy::prelude::{Rect, Vec2};
use bevy_egui::egui;
use editor_core::command_registry::OverlayState;
use viewport::{
    ViewportCameraMode, ViewportDebugSettings, ViewportInputState, ViewportRect, ViewportService,
    ViewportUiInput,
};

pub struct ViewportPanelInputs<'a> {
    pub overlays: &'a OverlayState,
    pub viewport_rect: &'a mut ViewportRect,
    pub viewport_service: &'a mut ViewportService,
    pub viewport_input: &'a ViewportUiInput,
    pub viewport_state: &'a ViewportInputState,
    pub camera_mode: &'a mut ViewportCameraMode,
    pub debug_settings: &'a mut ViewportDebugSettings,
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
    draw_viewport_header(ui, header_rect, inputs.camera_mode, inputs.debug_settings);
    update_viewport_rect_from_egui(
        body_rect,
        screen_rect,
        scale_factor,
        inputs.viewport_rect,
        inputs.viewport_service,
    );
    if inputs.overlays.show_overlays {
        draw_viewport_overlay(
            ui,
            body_rect,
            inputs.viewport_rect,
            inputs.viewport_input,
            inputs.viewport_state,
        );
    }
}

fn draw_viewport_header(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    camera_mode: &mut ViewportCameraMode,
    debug_settings: &mut ViewportDebugSettings,
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
            ui.checkbox(&mut debug_settings.show_ray_hit_marker, "Ray Hit")
                .on_hover_text("Draw a marker where the cursor ray hits the ground plane.");
            ui.checkbox(&mut debug_settings.show_prop_debug, "Prop Debug")
                .on_hover_text("Show a debug prop cube for picking tests.");
        });
    });
}

fn draw_viewport_overlay(
    ui: &mut egui::Ui,
    body_rect: egui::Rect,
    viewport_rect: &ViewportRect,
    viewport_input: &ViewportUiInput,
    viewport_state: &ViewportInputState,
) {
    if !egui_rect_is_finite(body_rect) {
        return;
    }
    let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 200, 255));
    ui.painter()
        .rect_stroke(body_rect, 0.0, stroke, egui::StrokeKind::Inside);

    let logical = viewport_rect.logical_origin;
    let logical_size = viewport_rect.logical_size;
    let physical = viewport_rect.physical_origin;
    let physical_size = viewport_rect.physical_size;
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
        viewport_rect.scale_factor,
        viewport_rect.is_valid
    );
    let ui_focus = viewport_input.wants_pointer || viewport_input.wants_keyboard;
    let focus_line = format!(
        "focus ui={} hover={} captured={}",
        ui_focus, viewport_state.hovered, viewport_state.captured
    );

    let font = egui::FontId::monospace(12.0);
    let color = egui::Color32::from_white_alpha(210);
    let start = body_rect.min + egui::vec2(6.0, 6.0);
    let painter = ui.painter();
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
        font,
        color,
    );
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
