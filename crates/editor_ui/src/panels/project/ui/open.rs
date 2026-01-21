use std::path::PathBuf;

use bevy_egui::egui;
use editor_core::project::ProjectCommand;

use super::super::ProjectPanelState;

pub(super) fn draw_open_project(ui: &mut egui::Ui, state: &mut ProjectPanelState) {
    ui.horizontal(|ui| {
        ui.label("Project folder");
        ui.text_edit_singleline(&mut state.open_project_path);
        if ui.button("Pick...").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                state.open_project_path = path.display().to_string();
            }
        }
    });

    let can_open = !state.open_project_path.trim().is_empty();
    if ui
        .add_enabled(can_open, egui::Button::new("Open Project"))
        .clicked()
    {
        let root = PathBuf::from(state.open_project_path.trim());
        state.pending_commands.push(ProjectCommand::Open { root });
    }
}
