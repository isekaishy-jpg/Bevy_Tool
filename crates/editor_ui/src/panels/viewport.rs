use bevy::prelude::{Rect, Vec2};
use bevy_egui::egui;
use editor_core::command_registry::OverlayState;
use viewport::{ViewportRect, ViewportService};

pub fn draw_viewport_panel(
    ui: &mut egui::Ui,
    overlays: &OverlayState,
    viewport_rect: &mut ViewportRect,
    viewport_service: &mut ViewportService,
) {
    let panel_rect = ui.max_rect();
    let screen_rect = ui.ctx().input(|input| input.content_rect());
    let scale_factor = ui.ctx().pixels_per_point();
    update_viewport_rect_from_egui(
        panel_rect,
        screen_rect,
        scale_factor,
        viewport_rect,
        viewport_service,
    );
    if overlays.show_overlays {
        if !egui_rect_is_finite(panel_rect) {
            return;
        }
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 200, 255));
        ui.painter()
            .rect_stroke(panel_rect, 0.0, stroke, egui::StrokeKind::Inside);
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
