use bevy::prelude::Resource;
use bevy_egui::egui;
use viewport::{
    OverlayPresentMode, SnapKind, ViewportDebugSettings, ViewportOverlaySettings,
    SUBGRID_SPACING_LEVELS,
};

#[derive(Resource, Default)]
pub struct ViewportOverlayPanelState {
    pub open: bool,
}

pub fn draw_overlay_options_window(
    ctx: &egui::Context,
    state: &mut ViewportOverlayPanelState,
    overlay_settings: &mut ViewportOverlaySettings,
    debug_settings: &mut ViewportDebugSettings,
) {
    if !state.open {
        return;
    }
    egui::Window::new("Overlay Options")
        .collapsible(false)
        .resizable(false)
        .open(&mut state.open)
        .show(ctx, |ui| {
            ui.label("Overlays");
            ui.checkbox(&mut overlay_settings.show_cursor_readout, "Cursor Readout");
            ui.checkbox(&mut overlay_settings.show_fps, "FPS");
            ui.label("Present Mode");
            egui::ComboBox::from_id_salt("present_mode")
                .selected_text(match overlay_settings.present_mode {
                    OverlayPresentMode::Vsync => "VSync",
                    OverlayPresentMode::AutoNoVsync => "AutoNoVsync",
                    OverlayPresentMode::Immediate => "Immediate",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut overlay_settings.present_mode,
                        OverlayPresentMode::Vsync,
                        "VSync",
                    );
                    ui.selectable_value(
                        &mut overlay_settings.present_mode,
                        OverlayPresentMode::AutoNoVsync,
                        "AutoNoVsync",
                    );
                    ui.selectable_value(
                        &mut overlay_settings.present_mode,
                        OverlayPresentMode::Immediate,
                        "Immediate",
                    );
                });
            ui.checkbox(&mut overlay_settings.show_tile_grid, "Tile Grid");
            ui.checkbox(&mut overlay_settings.show_chunk_grid, "Chunk Grid");
            ui.checkbox(&mut overlay_settings.show_subgrid, "Sub-grid");
            ui.checkbox(&mut overlay_settings.show_region_bounds, "Region Bounds");
            ui.checkbox(
                &mut overlay_settings.show_hover_highlight,
                "Hover Highlight",
            );
            ui.checkbox(
                &mut overlay_settings.show_selection_highlight,
                "Selection Highlight",
            );
            ui.checkbox(&mut overlay_settings.show_debug_markers, "Debug Markers");
            ui.checkbox(&mut overlay_settings.show_streaming, "Streaming States");
            ui.separator();
            ui.label("Snap");
            ui.horizontal(|ui| {
                ui.label("Mode");
                ui.selectable_value(&mut overlay_settings.snap_kind, SnapKind::Off, "Off");
                ui.selectable_value(&mut overlay_settings.snap_kind, SnapKind::Tile, "Tile");
                ui.selectable_value(&mut overlay_settings.snap_kind, SnapKind::Chunk, "Chunk");
                ui.selectable_value(
                    &mut overlay_settings.snap_kind,
                    SnapKind::Subgrid,
                    "Sub-grid",
                );
            });
            ui.horizontal(|ui| {
                ui.label("Sub-grid");
                egui::ComboBox::from_id_salt("subgrid_spacing")
                    .selected_text(format!("{} m", overlay_settings.subgrid_spacing))
                    .show_ui(ui, |ui| {
                        for spacing in SUBGRID_SPACING_LEVELS {
                            ui.selectable_value(
                                &mut overlay_settings.subgrid_spacing,
                                spacing,
                                format!("{spacing} m"),
                            );
                        }
                    });
            });
            ui.separator();
            ui.label("Debug");
            ui.checkbox(&mut debug_settings.show_ray_hit_marker, "Ray Hit")
                .on_hover_text("Draw a marker where the cursor ray hits the ground plane.");
            ui.checkbox(&mut debug_settings.show_prop_debug, "Prop Debug")
                .on_hover_text("Show a debug prop cube for picking tests.");
        });
}
