use bevy_egui::egui;
use editor_core::project::{NewProjectRequest, ProjectCommand};
use world::schema::RegionBounds;

use super::super::helpers::{can_create_project, resolve_project_root, world_spec_from_state};
use super::super::ProjectPanelState;

pub(super) fn draw_new_project(ui: &mut egui::Ui, state: &mut ProjectPanelState) {
    ui.horizontal(|ui| {
        ui.label("Project name");
        ui.text_edit_singleline(&mut state.new_project_name);
    });
    ui.horizontal(|ui| {
        ui.label("World name");
        ui.text_edit_singleline(&mut state.new_world_name);
    });
    ui.horizontal(|ui| {
        ui.label("Region id");
        ui.text_edit_singleline(&mut state.new_region_id);
    });
    ui.horizontal(|ui| {
        ui.label("Region name");
        ui.text_edit_singleline(&mut state.new_region_name);
    });
    ui.horizontal(|ui| {
        ui.label("Bounds min");
        ui.add(egui::DragValue::new(&mut state.region_min_x));
        ui.add(egui::DragValue::new(&mut state.region_min_y));
    });
    ui.horizontal(|ui| {
        ui.label("Bounds max");
        ui.add(egui::DragValue::new(&mut state.region_max_x));
        ui.add(egui::DragValue::new(&mut state.region_max_y));
    });

    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Project folder (optional)");
        ui.text_edit_singleline(&mut state.new_project_path);
        if ui.button("Pick...").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                state.new_project_path = path.display().to_string();
            }
        }
    });

    let can_create = can_create_project(state);
    if ui
        .add_enabled(can_create, egui::Button::new("Create Project"))
        .clicked()
    {
        let root = resolve_project_root(state);
        state.pending_commands.push(ProjectCommand::Create {
            root,
            request: NewProjectRequest {
                project_name: state.new_project_name.clone(),
                world_name: state.new_world_name.clone(),
                region_id: state.new_region_id.clone(),
                region_name: state.new_region_name.clone(),
                region_bounds: RegionBounds::new(
                    state.region_min_x,
                    state.region_min_y,
                    state.region_max_x,
                    state.region_max_y,
                ),
                world_spec: world_spec_from_state(state),
            },
        });
    }
}
