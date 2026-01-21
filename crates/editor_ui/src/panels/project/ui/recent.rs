use std::path::{Path, PathBuf};

use bevy_egui::egui;
use editor_core::prefs::EditorPrefs;
use editor_core::project::ProjectCommand;

use super::super::ProjectPanelState;

pub(super) fn draw_recent_projects(
    ui: &mut egui::Ui,
    state: &mut ProjectPanelState,
    prefs: &mut EditorPrefs,
) {
    if prefs.recent_projects.is_empty() {
        ui.label("No recent projects yet.");
    } else {
        let mut remove_index = None;
        for (index, recent) in prefs.recent_projects.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(&recent.display_name);
                ui.separator();
                ui.label(&recent.path);
                let exists = Path::new(&recent.path).exists();
                if !exists {
                    ui.colored_label(egui::Color32::LIGHT_RED, "Missing");
                }
                if ui.add_enabled(exists, egui::Button::new("Open")).clicked() {
                    state.pending_commands.push(ProjectCommand::Open {
                        root: PathBuf::from(&recent.path),
                    });
                }
                if ui.button("Remove").clicked() {
                    remove_index = Some(index);
                }
            });
        }

        if let Some(index) = remove_index {
            let path = prefs.recent_projects[index].path.clone();
            prefs.remove_recent(Path::new(&path));
        }
    }
}
