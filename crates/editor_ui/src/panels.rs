//! Panel stubs.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use editor_core::EditorConfig;
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
}

impl DockLayout {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Default for DockLayout {
    fn default() -> Self {
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

        Self { dock_state }
    }
}

struct EditorTabViewer<'a> {
    config: &'a EditorConfig,
}

impl<'a> TabViewer for EditorTabViewer<'a> {
    type Tab = PanelId;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            PanelId::Viewport => "Viewport".into(),
            PanelId::Assets => "Assets".into(),
            PanelId::Outliner => "Outliner".into(),
            PanelId::Inspector => "Inspector".into(),
            PanelId::World => "World".into(),
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
                ui.heading("World");
                ui.separator();
                ui.label(format!("Project: {}", self.config.project_name));
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

pub fn draw_root_panel(
    mut contexts: EguiContexts,
    config: Res<EditorConfig>,
    mut dock_layout: ResMut<DockLayout>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Bevy MMO World Editor");
            ui.separator();
            ui.label(format!("Project: {}", config.project_name));
            ui.separator();
            if ui.button("Reset Layout").clicked() {
                dock_layout.reset();
            }
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        let mut viewer = EditorTabViewer { config: &config };
        let style = Style::from_egui(ui.style().as_ref());
        DockArea::new(&mut dock_layout.dock_state)
            .style(style)
            .show_inside(ui, &mut viewer);
    });
}
