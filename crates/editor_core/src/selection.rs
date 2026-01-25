use bevy::log::info;
use bevy::prelude::*;
use foundation::ids::{InstanceId, TileCoord, TileId};

use crate::project::ActiveRegion;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionTarget {
    Tile {
        world_id: String,
        region_id: String,
        tile_id: TileId,
    },
    Prop {
        world_id: String,
        region_id: String,
        instance_id: InstanceId,
    },
}

#[derive(Resource, Debug, Default)]
pub struct SelectionState {
    pub selected: Option<SelectionTarget>,
}

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub enum SelectionCommand {
    SelectTile {
        world_id: String,
        region_id: String,
        tile_x: i32,
        tile_y: i32,
    },
    SelectProp {
        world_id: String,
        region_id: String,
        instance_id: InstanceId,
    },
    ClearSelection,
}

pub fn apply_selection_commands(
    event: On<SelectionCommand>,
    mut selection: ResMut<SelectionState>,
) {
    match event.event() {
        SelectionCommand::SelectTile {
            world_id,
            region_id,
            tile_x,
            tile_y,
        } => {
            let next = SelectionTarget::Tile {
                world_id: world_id.clone(),
                region_id: region_id.clone(),
                tile_id: TileId {
                    coord: TileCoord {
                        x: *tile_x,
                        y: *tile_y,
                    },
                },
            };
            if selection.selected.as_ref() != Some(&next) {
                selection.selected = Some(next.clone());
                info!(
                    "selection set: world={} region={} tile=({}, {})",
                    world_id, region_id, tile_x, tile_y
                );
            }
        }
        SelectionCommand::SelectProp {
            world_id,
            region_id,
            instance_id,
        } => {
            let next = SelectionTarget::Prop {
                world_id: world_id.clone(),
                region_id: region_id.clone(),
                instance_id: *instance_id,
            };
            if selection.selected.as_ref() != Some(&next) {
                selection.selected = Some(next);
                info!(
                    "selection set: world={} region={} prop={}",
                    world_id, region_id, instance_id.0
                );
            }
        }
        SelectionCommand::ClearSelection => {
            if selection.selected.take().is_some() {
                info!("selection cleared");
            }
        }
    }
}

pub fn clear_selection_on_region_change(
    active_region: Res<ActiveRegion>,
    selection: Res<SelectionState>,
    mut commands: Commands,
) {
    if !active_region.is_changed() {
        return;
    }
    let Some(selected) = selection.selected.as_ref() else {
        return;
    };
    match selected {
        SelectionTarget::Tile { region_id, .. } | SelectionTarget::Prop { region_id, .. } => {
            if active_region.region_id.as_deref() != Some(region_id.as_str()) {
                commands.trigger(SelectionCommand::ClearSelection);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clears_selection_when_active_region_changes() {
        let mut app = App::new();
        app.add_observer(apply_selection_commands);
        app.add_systems(Update, clear_selection_on_region_change);
        app.insert_resource(ActiveRegion {
            region_id: Some("region_a".to_string()),
        });
        app.insert_resource(SelectionState {
            selected: Some(SelectionTarget::Tile {
                world_id: "world_0".to_string(),
                region_id: "region_b".to_string(),
                tile_id: TileId {
                    coord: TileCoord { x: 0, y: 0 },
                },
            }),
        });

        app.update();

        let selection = app.world().resource::<SelectionState>();
        assert!(selection.selected.is_none());
    }
}
