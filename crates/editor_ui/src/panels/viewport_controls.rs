use bevy::ecs::message::MessageWriter;
use bevy::prelude::Resource;
use bevy_egui::egui;
use editor_core::project::ProjectState;
use viewport::{ViewportGoToTile, ViewportWorldSettings};

#[derive(Resource, Debug, Default)]
pub struct GoToTileState {
    pub open: bool,
    pub tile_x: i32,
    pub tile_y: i32,
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
        writer.write(ViewportGoToTile {
            tile_x: state.tile_x,
            tile_y: state.tile_y,
        });
        open = false;
    }

    if cancel {
        open = false;
    }

    state.open = open;
}
