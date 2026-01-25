use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::prelude::*;

use editor_core::project::ProjectState;
use editor_core::selection::SelectionCommand;
use viewport::{
    PropHoverState, ViewportCaptureSource, ViewportInputState, ViewportUiInput, WorldCursor,
};

#[derive(Resource, Debug, Default)]
pub struct SelectionInputState {
    pending_click: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn update_viewport_selection(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    ui_input: Res<ViewportUiInput>,
    input_state: Res<ViewportInputState>,
    cursor: Res<WorldCursor>,
    prop_hover: Res<PropHoverState>,
    project_state: Res<ProjectState>,
    mut state: ResMut<SelectionInputState>,
) {
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
                } else if cursor.has_hit {
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
