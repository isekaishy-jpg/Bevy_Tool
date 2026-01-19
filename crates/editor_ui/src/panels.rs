//! Panel stubs.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use editor_core::autosave::{restore_backup, AutosaveSettings, RecoveryState};
use editor_core::command_registry::CommandRegistry;
use editor_core::editor_state::ProjectEditorStateResource;
use editor_core::log_capture::LogBuffer;
use editor_core::prefs::EditorPrefs;
use editor_core::project::ProjectState;
use editor_core::EditorConfig;
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use serde::{Deserialize, Serialize};

pub mod command_palette;
pub mod logs;
pub mod project;
pub use command_palette::CommandPaletteState;
pub use logs::LogPanelState;
pub use project::ProjectPanelState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelId {
    Viewport,
    Assets,
    Outliner,
    Inspector,
    World,
    Console,
}

#[derive(Resource)]
pub struct DockLayout {
    dock_state: DockState<PanelId>,
    last_saved: Option<String>,
    loaded_project: Option<String>,
}

impl DockLayout {
    fn default_state() -> DockState<PanelId> {
        let mut dock_state = DockState::new(vec![PanelId::Viewport]);
        let tree = dock_state.main_surface_mut();
        let [center, _left] = tree.split_left(
            NodeIndex::root(),
            0.22,
            vec![PanelId::Assets, PanelId::Outliner],
        );
        let [center, _right] =
            tree.split_right(center, 0.25, vec![PanelId::Inspector, PanelId::World]);
        let [_center, _bottom] = tree.split_below(center, 0.28, vec![PanelId::Console]);

        dock_state
    }

    fn new_default() -> Self {
        Self {
            dock_state: Self::default_state(),
            last_saved: None,
            loaded_project: None,
        }
    }

    fn reset(&mut self) {
        self.dock_state = Self::default_state();
        self.last_saved = None;
    }
}

impl Default for DockLayout {
    fn default() -> Self {
        Self::new_default()
    }
}

struct EditorTabViewer<'a> {
    project_state: &'a ProjectState,
    log_buffer: Option<&'a LogBuffer>,
    config: &'a EditorConfig,
    prefs: &'a mut EditorPrefs,
    project_ui: &'a mut ProjectPanelState,
    log_ui: &'a mut LogPanelState,
}

impl<'a> TabViewer for EditorTabViewer<'a> {
    type Tab = PanelId;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            PanelId::Viewport => "Viewport".into(),
            PanelId::Assets => "Assets".into(),
            PanelId::Outliner => "Outliner".into(),
            PanelId::Inspector => "Inspector".into(),
            PanelId::World => "Project".into(),
            PanelId::Console => "Console".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            PanelId::Viewport => {
                ui.heading("Viewport");
                ui.label("3D viewport coming soon.");
            }
            PanelId::Assets => {
                ui.heading("Assets");
                ui.separator();
                ui.label("World/terrain tools pending.");
            }
            PanelId::Outliner => {
                ui.heading("Outliner");
                ui.separator();
                ui.label("Scene hierarchy will appear here.");
            }
            PanelId::Inspector => {
                ui.heading("Inspector");
                ui.separator();
                ui.label("Select an entity to inspect.");
            }
            PanelId::World => {
                project::draw_project_panel(ui, self.project_ui, self.project_state, self.prefs);
            }
            PanelId::Console => {
                logs::draw_log_panel(
                    ui,
                    self.log_ui,
                    self.log_buffer,
                    self.project_state,
                    self.config,
                );
            }
        }
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, PanelId::Viewport)
    }

    fn scroll_bars(&self, tab: &Self::Tab) -> [bool; 2] {
        if matches!(tab, PanelId::Viewport) {
            [false, false]
        } else {
            [true, true]
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_root_panel(
    mut contexts: EguiContexts,
    mut commands: Commands,
    config: Res<EditorConfig>,
    registry: Res<CommandRegistry>,
    mut project_state: ResMut<ProjectState>,
    log_buffer: Option<Res<LogBuffer>>,
    autosave_settings: Res<AutosaveSettings>,
    mut recovery_state: ResMut<RecoveryState>,
    mut prefs: ResMut<EditorPrefs>,
    mut project_ui: ResMut<ProjectPanelState>,
    mut palette_state: ResMut<CommandPaletteState>,
    mut dock_layout: ResMut<DockLayout>,
    mut editor_state: ResMut<ProjectEditorStateResource>,
    mut log_ui: ResMut<LogPanelState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    sync_layout_with_project(&project_state, &editor_state, &mut dock_layout);
    command_palette::handle_command_palette_shortcuts(ctx, &mut palette_state);

    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Bevy MMO World Editor");
            ui.separator();
            ui.label(format!(
                "Project: {} / World: {}",
                config.project_name, config.world_name
            ));
            ui.separator();
            if ui.button("Reset Layout").clicked() {
                dock_layout.reset();
                editor_state.state.dock_layout = None;
            }
            ui.separator();
            let autosave_label = format!(
                "Autosave ({:.0}s)",
                autosave_settings.interval.as_secs_f64()
            );
            ui.checkbox(&mut editor_state.state.autosave_enabled, autosave_label)
                .on_hover_text("Automatically saves project + world manifests.");
        });
        if let Some(backup) = recovery_state.pending_backup.clone() {
            if !recovery_state.dismissed {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Recovery available");
                    if ui.button("Restore autosave").clicked() {
                        if let Some(project) = &project_state.current {
                            match restore_backup(&project.root, &backup.path) {
                                Ok(_) => {
                                    recovery_state.dismissed = true;
                                    project_ui.pending_commands.push(
                                        editor_core::project::ProjectCommand::Open {
                                            root: project.root.clone(),
                                        },
                                    );
                                }
                                Err(err) => {
                                    project_state.last_error =
                                        Some(format!("restore failed: {err}"));
                                }
                            }
                        }
                    }
                    if ui.button("Dismiss").clicked() {
                        recovery_state.dismissed = true;
                    }
                });
            }
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Save").clicked() {
                    commands.trigger(editor_core::command_registry::CommandInvoked {
                        id: editor_core::command_registry::CommandId::Save,
                    });
                    ui.close();
                }
                if ui.button("Save All").clicked() {
                    commands.trigger(editor_core::command_registry::CommandInvoked {
                        id: editor_core::command_registry::CommandId::SaveAllDirty,
                    });
                    ui.close();
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    commands.trigger(editor_core::command_registry::CommandInvoked {
                        id: editor_core::command_registry::CommandId::Undo,
                    });
                    ui.close();
                }
                if ui.button("Redo").clicked() {
                    commands.trigger(editor_core::command_registry::CommandInvoked {
                        id: editor_core::command_registry::CommandId::Redo,
                    });
                    ui.close();
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("Focus Selection").clicked() {
                    commands.trigger(editor_core::command_registry::CommandInvoked {
                        id: editor_core::command_registry::CommandId::FocusSelection,
                    });
                    ui.close();
                }
                if ui.button("Toggle Overlays").clicked() {
                    commands.trigger(editor_core::command_registry::CommandInvoked {
                        id: editor_core::command_registry::CommandId::ToggleOverlays,
                    });
                    ui.close();
                }
            });
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        let mut viewer = EditorTabViewer {
            project_state: project_state.as_ref(),
            log_buffer: log_buffer.as_deref(),
            config: &config,
            prefs: &mut prefs,
            project_ui: &mut project_ui,
            log_ui: &mut log_ui,
        };
        let style = Style::from_egui(ui.style().as_ref());
        DockArea::new(&mut dock_layout.dock_state)
            .style(style)
            .show_inside(ui, &mut viewer);
    });

    persist_layout(&mut editor_state, &mut dock_layout);
    for command in project_ui.pending_commands.drain(..) {
        commands.trigger(command);
    }

    command_palette::draw_command_palette(ctx, &mut palette_state, &registry, &mut commands);
}

fn persist_layout(editor_state: &mut ProjectEditorStateResource, dock_layout: &mut DockLayout) {
    if editor_state.root.is_none() {
        return;
    }
    let Ok(serialized) = serde_json::to_string(&dock_layout.dock_state) else {
        return;
    };
    if dock_layout.last_saved.as_deref() == Some(&serialized) {
        return;
    }
    dock_layout.last_saved = Some(serialized.clone());
    editor_state.state.dock_layout = Some(serialized);
}

fn sync_layout_with_project(
    project_state: &ProjectState,
    editor_state: &ProjectEditorStateResource,
    dock_layout: &mut DockLayout,
) {
    let project_key = project_state
        .current
        .as_ref()
        .map(|info| info.root.to_string_lossy().to_string());

    if dock_layout.loaded_project == project_key {
        return;
    }

    dock_layout.loaded_project = project_key.clone();
    if let Some(layout) = editor_state.state.dock_layout.as_deref() {
        if let Ok(dock_state) = serde_json::from_str::<DockState<PanelId>>(layout) {
            dock_layout.dock_state = dock_state;
            dock_layout.last_saved = Some(layout.to_string());
            return;
        }
    }

    dock_layout.reset();
}
