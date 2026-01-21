use bevy::prelude::{Rect, Vec2};
use bevy_egui::egui;
use editor_core::command_registry::OverlayState;
use viewport::{ViewportCameraMode, ViewportRect, ViewportService};

pub fn draw_viewport_panel(
    ui: &mut egui::Ui,
    overlays: &OverlayState,
    viewport_rect: &mut ViewportRect,
    viewport_service: &mut ViewportService,
    camera_mode: &mut ViewportCameraMode,
) {
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
    draw_viewport_header(ui, header_rect, camera_mode);
    update_viewport_rect_from_egui(
        body_rect,
        screen_rect,
        scale_factor,
        viewport_rect,
        viewport_service,
    );
    if overlays.show_overlays {
        if !egui_rect_is_finite(body_rect) {
            return;
        }
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 200, 255));
        ui.painter()
            .rect_stroke(body_rect, 0.0, stroke, egui::StrokeKind::Inside);
    }
}

fn draw_viewport_header(ui: &mut egui::Ui, rect: egui::Rect, camera_mode: &mut ViewportCameraMode) {
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
        });
    });
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
