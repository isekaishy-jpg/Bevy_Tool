use bevy::prelude::Resource;
use world::schema::DEFAULT_WORLD_SPEC;

pub struct NewWorldState {
    pub name: String,
    pub region_id: String,
    pub region_name: String,
    pub region_min_x: i32,
    pub region_min_y: i32,
    pub region_max_x: i32,
    pub region_max_y: i32,
    pub tile_size_meters: f32,
    pub chunks_per_tile: u16,
    pub heightfield_samples: u16,
    pub weightmap_resolution: u16,
    pub liquids_resolution: u16,
}

impl Default for NewWorldState {
    fn default() -> Self {
        Self {
            name: "NewWorld".to_string(),
            region_id: "r000".to_string(),
            region_name: "Region 0".to_string(),
            region_min_x: 0,
            region_min_y: 0,
            region_max_x: 255,
            region_max_y: 255,
            tile_size_meters: DEFAULT_WORLD_SPEC.tile_size_meters,
            chunks_per_tile: DEFAULT_WORLD_SPEC.chunks_per_tile,
            heightfield_samples: DEFAULT_WORLD_SPEC.heightfield_samples,
            weightmap_resolution: DEFAULT_WORLD_SPEC.weightmap_resolution,
            liquids_resolution: DEFAULT_WORLD_SPEC.liquids_resolution,
        }
    }
}

#[derive(Resource)]
pub struct ProjectPanelState {
    pub new_project_name: String,
    pub new_world_name: String,
    pub new_region_id: String,
    pub new_region_name: String,
    pub new_project_path: String,
    pub open_project_path: String,
    pub project_key: Option<String>,
    pub world_key: Option<String>,
    pub project_name_edit: String,
    pub world_name_edit: String,
    pub tile_size_meters: f32,
    pub chunks_per_tile: u16,
    pub heightfield_samples: u16,
    pub weightmap_resolution: u16,
    pub liquids_resolution: u16,
    pub region_min_x: i32,
    pub region_min_y: i32,
    pub region_max_x: i32,
    pub region_max_y: i32,
    pub new_world: NewWorldState,
    pub pending_commands: Vec<editor_core::project::ProjectCommand>,
}

impl Default for ProjectPanelState {
    fn default() -> Self {
        Self {
            new_project_name: "NewProject".to_string(),
            new_world_name: "NewWorld".to_string(),
            new_region_id: "r000".to_string(),
            new_region_name: "Region 0".to_string(),
            new_project_path: String::new(),
            open_project_path: String::new(),
            project_key: None,
            world_key: None,
            project_name_edit: String::new(),
            world_name_edit: String::new(),
            tile_size_meters: DEFAULT_WORLD_SPEC.tile_size_meters,
            chunks_per_tile: DEFAULT_WORLD_SPEC.chunks_per_tile,
            heightfield_samples: DEFAULT_WORLD_SPEC.heightfield_samples,
            weightmap_resolution: DEFAULT_WORLD_SPEC.weightmap_resolution,
            liquids_resolution: DEFAULT_WORLD_SPEC.liquids_resolution,
            region_min_x: 0,
            region_min_y: 0,
            region_max_x: 255,
            region_max_y: 255,
            new_world: NewWorldState::default(),
            pending_commands: Vec::new(),
        }
    }
}
