use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::prelude::*;

use editor_core::project::ProjectState;
use editor_core::selection::{SelectionCommand, SelectionState, SelectionTarget};
use viewport::{
    PropHoverState, ViewportCaptureSource, ViewportInputState, ViewportSelectionState,
    ViewportUiInput, WorldCursor,
};

#[derive(Resource, Debug, Default)]
pub struct SelectionInputState {
    pending_click: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn update_viewport_selection(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    ui_input: Res<ViewportUiInput>,
    input_state: Res<ViewportInputState>,
    cursor: Res<WorldCursor>,
    prop_hover: Res<PropHoverState>,
    project_state: Res<ProjectState>,
    mut state: ResMut<SelectionInputState>,
) {
    if keys.just_pressed(KeyCode::Escape) && input_state.hotkeys_allowed {
        commands.trigger(SelectionCommand::ClearSelection);
        state.pending_click = false;
    }

    if input_state.captured && input_state.captor == Some(ViewportCaptureSource::Tool) {
        state.pending_click = false;
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Left)
        && input_state.hovered
        && !ui_input.wants_pointer
    {
        state.pending_click = true;
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        if state.pending_click {
            let world_id = project_state
                .current
                .as_ref()
                .and_then(|project| project.current_world_id.clone());
            let region_id = cursor.region_id.clone();

            if let (Some(world_id), Some(region_id)) = (world_id, region_id) {
                if let Some(hit) = prop_hover.hovered.as_ref() {
                    commands.trigger(SelectionCommand::SelectProp {
                        world_id,
                        region_id,
                        instance_id: hit.instance_id,
                    });
                } else if cursor.has_hit && cursor.in_bounds {
                    commands.trigger(SelectionCommand::SelectTile {
                        world_id,
                        region_id,
                        tile_x: cursor.tile_x,
                        tile_y: cursor.tile_y,
                    });
                } else {
                    commands.trigger(SelectionCommand::ClearSelection);
                }
            } else {
                commands.trigger(SelectionCommand::ClearSelection);
            }
        }
        state.pending_click = false;
    }

    if !mouse_buttons.pressed(MouseButton::Left) && !input_state.hovered {
        state.pending_click = false;
    }
}

pub fn sync_viewport_selection_overlay(
    selection: Res<SelectionState>,
    mut overlay_selection: ResMut<ViewportSelectionState>,
) {
    let mut selected_tile = None;
    let mut selected_prop = None;
    if let Some(target) = selection.selected.as_ref() {
        match target {
            SelectionTarget::Tile { tile_id, .. } => {
                selected_tile = Some(tile_id.coord);
            }
            SelectionTarget::Prop { instance_id, .. } => {
                selected_prop = Some(*instance_id);
            }
        }
    }

    if overlay_selection.selected_tile != selected_tile
        || overlay_selection.selected_prop != selected_prop
    {
        overlay_selection.selected_tile = selected_tile;
        overlay_selection.selected_prop = selected_prop;
    }
}
