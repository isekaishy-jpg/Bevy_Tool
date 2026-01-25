//! Panel stubs.

use ::viewport::{
    ViewportCameraMode, ViewportDebugSettings, ViewportFocusRequest, ViewportGoToTile,
    ViewportInputState, ViewportRect, ViewportService, ViewportUiInput, ViewportWorldSettings,
};
use bevy::ecs::message::MessageWriter;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use editor_core::autosave::{restore_backup, AutosaveSettings, RecoveryState};
use editor_core::command_registry::{CommandRegistry, FocusSelectionRequest, OverlayState};
use editor_core::editor_state::ProjectEditorStateResource;
use editor_core::log_capture::LogBuffer;
use editor_core::prefs::EditorPrefs;
use editor_core::project::{ActiveRegion, ProjectState};
use editor_core::EditorConfig;
use egui_dock::{DockArea, Style, TabViewer};
use serde::{Deserialize, Serialize};

#[derive(SystemParam)]
pub(crate) struct ViewportUiParams<'w> {
    viewport_rect: ResMut<'w, ViewportRect>,
    viewport_service: ResMut<'w, ViewportService>,
    viewport_input: ResMut<'w, ViewportUiInput>,
    viewport_state: Res<'w, ViewportInputState>,
    viewport_world: ResMut<'w, ViewportWorldSettings>,
    camera_mode: ResMut<'w, ViewportCameraMode>,
    viewport_debug: ResMut<'w, ViewportDebugSettings>,
    go_to_state: ResMut<'w, GoToTileState>,
    go_to_writer: MessageWriter<'w, ViewportGoToTile>,
    focus_writer: MessageWriter<'w, ViewportFocusRequest>,
    focus_request: ResMut<'w, FocusSelectionRequest>,
}

#[derive(SystemParam)]
pub(crate) struct ProjectUiParams<'w> {
    project_state: ResMut<'w, ProjectState>,
    active_region: ResMut<'w, ActiveRegion>,
    prefs: ResMut<'w, EditorPrefs>,
    project_ui: ResMut<'w, ProjectPanelState>,
}

pub mod command_palette;
pub mod layout;
pub mod logs;
pub mod project;
pub mod viewport;
pub mod viewport_controls;
pub use command_palette::CommandPaletteState;
pub use layout::DockLayout;
pub use logs::LogPanelState;
pub use project::ProjectPanelState;
pub use viewport_controls::GoToTileState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelId {
    Viewport,
    Assets,
    Outliner,
    Inspector,
    World,
    Console,
}

struct EditorTabViewer<'a> {
    project_state: &'a ProjectState,
    log_buffer: Option<&'a LogBuffer>,
    config: &'a EditorConfig,
    prefs: &'a mut EditorPrefs,
    project_ui: &'a mut ProjectPanelState,
    active_region: &'a mut ActiveRegion,
    log_ui: &'a mut LogPanelState,
    overlays: &'a OverlayState,
    viewport_rect: &'a mut ViewportRect,
    viewport_service: &'a mut ViewportService,
    viewport_input: &'a ViewportUiInput,
    viewport_state: &'a ViewportInputState,
    camera_mode: &'a mut ViewportCameraMode,
    viewport_debug: &'a mut ViewportDebugSettings,
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
                let mut inputs = viewport::ViewportPanelInputs {
                    overlays: self.overlays,
                    viewport_rect: self.viewport_rect,
                    viewport_service: self.viewport_service,
                    viewport_input: self.viewport_input,
                    viewport_state: self.viewport_state,
                    camera_mode: self.camera_mode,
                    debug_settings: self.viewport_debug,
                };
                viewport::draw_viewport_panel(ui, &mut inputs);
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
                project::draw_project_panel(
                    ui,
                    self.project_ui,
                    self.project_state,
                    self.prefs,
                    self.active_region,
                );
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

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, PanelId::Viewport)
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_root_panel(
    mut contexts: EguiContexts,
    mut commands: Commands,
    config: Res<EditorConfig>,
    registry: Res<CommandRegistry>,
    overlays: Res<OverlayState>,
    log_buffer: Option<Res<LogBuffer>>,
    autosave_settings: Res<AutosaveSettings>,
    mut recovery_state: ResMut<RecoveryState>,
    mut palette_state: ResMut<CommandPaletteState>,
    mut dock_layout: ResMut<DockLayout>,
    mut editor_state: ResMut<ProjectEditorStateResource>,
    mut log_ui: ResMut<LogPanelState>,
    mut project: ProjectUiParams,
    mut viewport: ViewportUiParams,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    viewport.viewport_input.wants_pointer = ctx.is_using_pointer();
    viewport.viewport_input.wants_keyboard = ctx.wants_keyboard_input();
    viewport_controls::sync_world_settings(&project.project_state, &mut viewport.viewport_world);
    viewport_controls::handle_go_to_shortcut(ctx, &mut viewport.go_to_state);
    if viewport.focus_request.requested {
        viewport
            .focus_writer
            .write(ViewportFocusRequest { world_point: None });
        viewport.focus_request.requested = false;
    }

    layout::sync_layout_with_project(&project.project_state, &editor_state, &mut dock_layout);
    command_palette::handle_command_palette_shortcuts(ctx, &mut palette_state);
    viewport.viewport_rect.invalidate();
    viewport.viewport_service.rect = *viewport.viewport_rect;

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
                        if let Some(current_project) = &project.project_state.current {
                            match restore_backup(&current_project.root, &backup.path) {
                                Ok(_) => {
                                    recovery_state.dismissed = true;
                                    project.project_ui.pending_commands.push(
                                        editor_core::project::ProjectCommand::Open {
                                            root: current_project.root.clone(),
                                        },
                                    );
                                }
                                Err(err) => {
                                    project.project_state.last_error =
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
                viewport_controls::draw_go_to_menu(ui, &mut viewport.go_to_state);
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

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            let mut viewer = EditorTabViewer {
                project_state: project.project_state.as_ref(),
                log_buffer: log_buffer.as_deref(),
                config: &config,
                prefs: &mut project.prefs,
                project_ui: &mut project.project_ui,
                active_region: &mut project.active_region,
                log_ui: &mut log_ui,
                overlays: overlays.as_ref(),
                viewport_rect: &mut viewport.viewport_rect,
                viewport_service: &mut viewport.viewport_service,
                viewport_input: &viewport.viewport_input,
                viewport_state: &viewport.viewport_state,
                camera_mode: &mut viewport.camera_mode,
                viewport_debug: &mut viewport.viewport_debug,
            };
            let style = Style::from_egui(ui.style().as_ref());
            DockArea::new(&mut dock_layout.dock_state)
                .style(style)
                .show_inside(ui, &mut viewer);
        });

    layout::persist_layout(&mut editor_state, &mut dock_layout);
    for command in project.project_ui.pending_commands.drain(..) {
        commands.trigger(command);
    }

    command_palette::draw_command_palette(ctx, &mut palette_state, &registry, &mut commands);
    let active_region_ref = project.active_region.as_ref();
    viewport_controls::draw_go_to_modal(
        ctx,
        &mut viewport.go_to_state,
        &mut viewport.go_to_writer,
        &project.project_state,
        active_region_ref,
    );
}
