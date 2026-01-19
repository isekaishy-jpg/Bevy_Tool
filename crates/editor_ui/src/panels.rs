//! Panel stubs.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use editor_core::command_registry::CommandRegistry;
use editor_core::prefs::EditorPrefs;
use editor_core::project::ProjectState;
use editor_core::EditorConfig;
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use serde::{Deserialize, Serialize};

pub mod command_palette;
pub mod project;
pub use command_palette::CommandPaletteState;
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
        }
    }

    fn reset(&mut self) {
        self.dock_state = Self::default_state();
        self.last_saved = None;
    }
}

impl FromWorld for DockLayout {
    fn from_world(world: &mut World) -> Self {
        let Some(prefs) = world.get_resource::<EditorPrefs>() else {
            return Self::new_default();
        };
        let Some(layout) = prefs.dock_layout.as_deref() else {
            return Self::new_default();
        };
        match serde_json::from_str::<DockState<PanelId>>(layout) {
            Ok(dock_state) => Self {
                dock_state,
                last_saved: Some(layout.to_string()),
            },
            Err(_) => Self::new_default(),
        }
    }
}

struct EditorTabViewer<'a> {
    project_state: &'a ProjectState,
    prefs: &'a mut EditorPrefs,
    project_ui: &'a mut ProjectPanelState,
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
                ui.heading("Console");
                ui.separator();
                ui.label("Logs and diagnostics pending.");
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
    project_state: Res<ProjectState>,
    mut prefs: ResMut<EditorPrefs>,
    mut project_ui: ResMut<ProjectPanelState>,
    mut palette_state: ResMut<CommandPaletteState>,
    mut dock_layout: ResMut<DockLayout>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    command_palette::handle_command_palette_shortcuts(ctx, &mut palette_state);

    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Bevy MMO World Editor");
            ui.separator();
            ui.label(format!("Project: {}", config.project_name));
            ui.separator();
            if ui.button("Reset Layout").clicked() {
                dock_layout.reset();
                prefs.dock_layout = None;
            }
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        let mut viewer = EditorTabViewer {
            project_state: &project_state,
            prefs: &mut prefs,
            project_ui: &mut project_ui,
        };
        let style = Style::from_egui(ui.style().as_ref());
        DockArea::new(&mut dock_layout.dock_state)
            .style(style)
            .show_inside(ui, &mut viewer);
    });

    persist_layout(&mut prefs, &mut dock_layout);
    for command in project_ui.pending_commands.drain(..) {
        commands.trigger(command);
    }

    command_palette::draw_command_palette(ctx, &mut palette_state, &registry, &mut commands);
}

fn persist_layout(prefs: &mut EditorPrefs, dock_layout: &mut DockLayout) {
    let Ok(serialized) = serde_json::to_string(&dock_layout.dock_state) else {
        return;
    };
    if dock_layout.last_saved.as_deref() == Some(&serialized) {
        return;
    }
    dock_layout.last_saved = Some(serialized.clone());
    prefs.dock_layout = Some(serialized);
}
