use bevy::prelude::*;
use egui_dock::{DockState, NodeIndex};

use super::PanelId;
use editor_core::editor_state::ProjectEditorStateResource;
use editor_core::project::ProjectState;

#[derive(Resource)]
pub struct DockLayout {
    pub(super) dock_state: DockState<PanelId>,
    pub(super) last_saved: Option<String>,
    pub(super) loaded_project: Option<String>,
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

    pub fn reset(&mut self) {
        self.dock_state = Self::default_state();
        self.last_saved = None;
    }
}

impl Default for DockLayout {
    fn default() -> Self {
        Self::new_default()
    }
}

pub(super) fn persist_layout(
    editor_state: &mut ProjectEditorStateResource,
    dock_layout: &mut DockLayout,
) {
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

pub(super) fn sync_layout_with_project(
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
