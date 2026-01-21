use bevy_egui::egui;
use editor_core::prefs::EditorPrefs;
use editor_core::project::ProjectState;

use super::ProjectPanelState;

mod active;
mod create;
mod open;
mod recent;

pub fn draw_project_panel(
    ui: &mut egui::Ui,
    state: &mut ProjectPanelState,
    project_state: &ProjectState,
    prefs: &mut EditorPrefs,
) {
    ui.heading("Project");
    ui.separator();

    if let Some(error) = &project_state.last_error {
        ui.colored_label(egui::Color32::LIGHT_RED, error);
        ui.separator();
    }

    if let Some(info) = &project_state.current {
        active::draw_active_project(ui, state, info);
        ui.separator();
    } else {
        ui.label("No project open.");
        ui.separator();
    }

    ui.collapsing("New Project", |ui| {
        create::draw_new_project(ui, state);
    });

    ui.collapsing("Open Project", |ui| {
        open::draw_open_project(ui, state);
    });

    ui.separator();
    ui.heading("Recent Projects");
    recent::draw_recent_projects(ui, state, prefs);
}
