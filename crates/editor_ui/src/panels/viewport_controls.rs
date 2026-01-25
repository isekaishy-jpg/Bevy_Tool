use bevy::ecs::message::MessageWriter;
use bevy::log::{info, warn};
use bevy::prelude::Resource;
use bevy_egui::egui;
use editor_core::project::{ActiveRegion, ProjectState};
use viewport::{ViewportGoToTile, ViewportWorldSettings};
use world::schema::RegionBounds;

#[derive(Resource, Debug, Default)]
pub struct GoToTileState {
    pub open: bool,
    pub tile_x: i32,
    pub tile_y: i32,
    pub last_error: Option<String>,
    pub last_warning: Option<String>,
}

pub fn sync_world_settings(
    project_state: &ProjectState,
    world_settings: &mut ViewportWorldSettings,
) {
    if let Some(project) = &project_state.current {
        if let Some(world) = project.current_world() {
            world_settings.tile_size_meters = world.manifest.world_spec.tile_size_meters;
        }
    }
}

pub fn handle_go_to_shortcut(ctx: &egui::Context, state: &mut GoToTileState) {
    if ctx.input(|input| input.key_pressed(egui::Key::G)) && !ctx.wants_keyboard_input() {
        state.open = true;
    }
}

pub fn draw_go_to_menu(ui: &mut egui::Ui, state: &mut GoToTileState) {
    if ui.button("Go To Tile...").clicked() {
        state.open = true;
        ui.close();
    }
}

pub fn draw_go_to_modal(
    ctx: &egui::Context,
    state: &mut GoToTileState,
    writer: &mut MessageWriter<ViewportGoToTile>,
    project_state: &ProjectState,
    active_region: &ActiveRegion,
) {
    if !state.open {
        return;
    }

    let mut open = state.open;
    let mut submit = false;
    let mut cancel = false;
    egui::Window::new("Go To Tile")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label("Enter tile coordinates");
            ui.horizontal(|ui| {
                ui.label("Tile X");
                ui.add(egui::DragValue::new(&mut state.tile_x).speed(1));
            });
            ui.horizontal(|ui| {
                ui.label("Tile Y");
                ui.add(egui::DragValue::new(&mut state.tile_y).speed(1));
            });

            if let Some(error) = &state.last_error {
                ui.colored_label(egui::Color32::LIGHT_RED, error);
            } else if let Some(warning) = &state.last_warning {
                ui.colored_label(egui::Color32::YELLOW, warning);
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Go").clicked() {
                    submit = true;
                }
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
            });

            submit |= ui.input(|input| input.key_pressed(egui::Key::Enter));
            cancel |= ui.input(|input| input.key_pressed(egui::Key::Escape));
        });

    if submit {
        state.last_error = None;
        state.last_warning = None;
        if let Some(bounds) = active_region_bounds(project_state, active_region) {
            let (clamped_x, clamped_y) = clamp_to_bounds(state.tile_x, state.tile_y, bounds);
            if clamped_x != state.tile_x || clamped_y != state.tile_y {
                let message = format!(
                    "Clamped to region bounds [{}, {}] -> [{}, {}]",
                    bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y
                );
                state.last_warning = Some(message.clone());
                info!("go to tile clamped: {message}");
            }
            state.tile_x = clamped_x;
            state.tile_y = clamped_y;
            writer.write(ViewportGoToTile {
                tile_x: state.tile_x,
                tile_y: state.tile_y,
            });
            open = false;
        } else {
            let message = "Go To Tile unavailable: no active region selected.".to_string();
            state.last_error = Some(message.clone());
            warn!("{message}");
        }
    }

    if cancel {
        open = false;
    }

    state.open = open;
}

fn active_region_bounds(
    project_state: &ProjectState,
    active_region: &ActiveRegion,
) -> Option<RegionBounds> {
    let project = project_state.current.as_ref()?;
    let world = project.current_world()?;
    let regions = &world.manifest.regions;
    if regions.is_empty() {
        return None;
    }

    if let Some(active_id) = active_region.region_id.as_deref() {
        if let Some(region) = regions.iter().find(|region| region.region_id == active_id) {
            return Some(region.bounds);
        }
    }

    Some(regions[0].bounds)
}

fn clamp_to_bounds(tile_x: i32, tile_y: i32, bounds: RegionBounds) -> (i32, i32) {
    let clamped_x = tile_x.clamp(bounds.min_x, bounds.max_x);
    let clamped_y = tile_y.clamp(bounds.min_y, bounds.max_y);
    (clamped_x, clamped_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_to_bounds_limits_out_of_range_values() {
        let bounds = RegionBounds::new(0, 0, 4, 4);
        let (x, y) = clamp_to_bounds(-3, 9, bounds);
        assert_eq!((x, y), (0, 4));
    }

    #[test]
    fn clamp_to_bounds_preserves_in_range_values() {
        let bounds = RegionBounds::new(-2, -2, 2, 2);
        let (x, y) = clamp_to_bounds(1, -1, bounds);
        assert_eq!((x, y), (1, -1));
    }
}
