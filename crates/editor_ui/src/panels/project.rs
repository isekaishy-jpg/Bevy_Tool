use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_egui::egui;
use editor_core::prefs::EditorPrefs;
use editor_core::project::{NewProjectRequest, ProjectCommand, ProjectState};
use world::schema::{RegionBounds, RegionManifest, WorldSpec, DEFAULT_WORLD_SPEC};

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
    pub pending_commands: Vec<ProjectCommand>,
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
            pending_commands: Vec::new(),
        }
    }
}

pub fn draw_project_panel(
    ui: &mut egui::Ui,
    state: &mut ProjectPanelState,
    project_state: &ProjectState,
    prefs: &mut EditorPrefs,
) {
    ui.heading("Project");
    ui.separator();

    if let Some(error) = &project_state.last_error {
        ui.colored_label(egui::Color32::LIGHT_RED, error);
        ui.separator();
    }

    if let Some(info) = &project_state.current {
        sync_from_project(state, info);

        ui.label(format!("Root: {}", info.root.display()));
        ui.separator();

        ui.heading("Project Settings");
        ui.horizontal(|ui| {
            ui.label("Project name");
            ui.text_edit_singleline(&mut state.project_name_edit);
        });

        let project_dirty = state.project_name_edit.trim() != info.manifest.project_name;
        if ui
            .add_enabled(project_dirty, egui::Button::new("Save Project"))
            .clicked()
        {
            let mut updated = info.manifest.clone();
            let project_name = state.project_name_edit.trim();
            updated.project_name = if project_name.is_empty() {
                info.manifest.project_name.clone()
            } else {
                project_name.to_string()
            };
            state
                .pending_commands
                .push(ProjectCommand::UpdateProjectManifest {
                    root: info.root.clone(),
                    manifest: updated,
                });
        }

        if let Some(world) = info.current_world() {
            ui.separator();
            ui.heading("World Settings");

            let mut selected_world = info.current_world_id.clone().unwrap_or_default();
            egui::ComboBox::from_label("World")
                .selected_text(&world.manifest.world_name)
                .show_ui(ui, |ui| {
                    for entry in &info.worlds {
                        let label = format!(
                            "{} ({})",
                            entry.manifest.world_name, entry.manifest.world_id
                        );
                        if ui
                            .selectable_value(
                                &mut selected_world,
                                entry.manifest.world_id.clone(),
                                label,
                            )
                            .clicked()
                        {
                            state
                                .pending_commands
                                .push(ProjectCommand::SetCurrentWorld {
                                    world_id: selected_world.clone(),
                                });
                        }
                    }
                });

            ui.horizontal(|ui| {
                ui.label("World name");
                ui.text_edit_singleline(&mut state.world_name_edit);
            });

            let spec_locked = world.has_tiles;
            if spec_locked {
                ui.label("World spec locked because tiles exist on disk.");
            }

            ui.add_enabled_ui(!spec_locked, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Tile size (m)");
                    ui.add(egui::DragValue::new(&mut state.tile_size_meters).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Chunks per tile");
                    ui.add(egui::DragValue::new(&mut state.chunks_per_tile).range(1..=64));
                });
                ui.horizontal(|ui| {
                    ui.label("Heightfield samples");
                    ui.add(egui::DragValue::new(&mut state.heightfield_samples).range(2..=4096));
                });
                ui.horizontal(|ui| {
                    ui.label("Weightmap resolution");
                    ui.add(egui::DragValue::new(&mut state.weightmap_resolution).range(2..=4096));
                });
                ui.horizontal(|ui| {
                    ui.label("Liquids resolution");
                    ui.add(egui::DragValue::new(&mut state.liquids_resolution).range(2..=4096));
                });
            });

            let world_dirty = is_world_dirty(state, world);
            if ui
                .add_enabled(world_dirty, egui::Button::new("Save World"))
                .clicked()
            {
                let updated = world_manifest_from_state(state, world);
                state
                    .pending_commands
                    .push(ProjectCommand::UpdateWorldManifest {
                        root: info.root.clone(),
                        manifest: updated,
                    });
            }

            ui.separator();
            ui.heading("Regions");
            if world.manifest.regions.is_empty() {
                ui.label("No regions yet.");
            } else {
                for region in &world.manifest.regions {
                    ui.horizontal(|ui| {
                        ui.label(&region.name);
                        ui.separator();
                        ui.label(&region.region_id);
                        ui.separator();
                        ui.label(format!(
                            "bounds [{}, {}] -> [{}, {}]",
                            region.bounds.min_x,
                            region.bounds.min_y,
                            region.bounds.max_x,
                            region.bounds.max_y
                        ));
                    });
                }
            }

            ui.separator();
            ui.heading("Add Region");
            ui.horizontal(|ui| {
                ui.label("Region id");
                ui.text_edit_singleline(&mut state.new_region_id);
            });
            ui.horizontal(|ui| {
                ui.label("Region name");
                ui.text_edit_singleline(&mut state.new_region_name);
            });
            ui.horizontal(|ui| {
                ui.label("Bounds min");
                ui.add(egui::DragValue::new(&mut state.region_min_x));
                ui.add(egui::DragValue::new(&mut state.region_min_y));
            });
            ui.horizontal(|ui| {
                ui.label("Bounds max");
                ui.add(egui::DragValue::new(&mut state.region_max_x));
                ui.add(egui::DragValue::new(&mut state.region_max_y));
            });

            let can_add_region = can_add_region(state, world);
            if ui
                .add_enabled(can_add_region, egui::Button::new("Create Region"))
                .clicked()
            {
                let mut updated = world.manifest.clone();
                updated.regions.push(RegionManifest {
                    region_id: state.new_region_id.trim().to_string(),
                    name: state.new_region_name.trim().to_string(),
                    bounds: RegionBounds::new(
                        state.region_min_x,
                        state.region_min_y,
                        state.region_max_x,
                        state.region_max_y,
                    ),
                });
                state
                    .pending_commands
                    .push(ProjectCommand::UpdateWorldManifest {
                        root: info.root.clone(),
                        manifest: updated,
                    });
            }
        }

        ui.separator();
    } else {
        ui.label("No project open.");
        ui.separator();
    }

    ui.collapsing("New Project", |ui| {
        ui.horizontal(|ui| {
            ui.label("Project name");
            ui.text_edit_singleline(&mut state.new_project_name);
        });
        ui.horizontal(|ui| {
            ui.label("World name");
            ui.text_edit_singleline(&mut state.new_world_name);
        });
        ui.horizontal(|ui| {
            ui.label("Region id");
            ui.text_edit_singleline(&mut state.new_region_id);
        });
        ui.horizontal(|ui| {
            ui.label("Region name");
            ui.text_edit_singleline(&mut state.new_region_name);
        });
        ui.horizontal(|ui| {
            ui.label("Bounds min");
            ui.add(egui::DragValue::new(&mut state.region_min_x));
            ui.add(egui::DragValue::new(&mut state.region_min_y));
        });
        ui.horizontal(|ui| {
            ui.label("Bounds max");
            ui.add(egui::DragValue::new(&mut state.region_max_x));
            ui.add(egui::DragValue::new(&mut state.region_max_y));
        });

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Project folder (optional)");
            ui.text_edit_singleline(&mut state.new_project_path);
            if ui.button("Pick...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    state.new_project_path = path.display().to_string();
                }
            }
        });

        let can_create = can_create_project(state);
        if ui
            .add_enabled(can_create, egui::Button::new("Create Project"))
            .clicked()
        {
            let root = resolve_project_root(state);
            state.pending_commands.push(ProjectCommand::Create {
                root,
                request: NewProjectRequest {
                    project_name: state.new_project_name.clone(),
                    world_name: state.new_world_name.clone(),
                    region_id: state.new_region_id.clone(),
                    region_name: state.new_region_name.clone(),
                    region_bounds: RegionBounds::new(
                        state.region_min_x,
                        state.region_min_y,
                        state.region_max_x,
                        state.region_max_y,
                    ),
                    world_spec: world_spec_from_state(state),
                },
            });
        }
    });

    ui.collapsing("Open Project", |ui| {
        ui.horizontal(|ui| {
            ui.label("Project folder");
            ui.text_edit_singleline(&mut state.open_project_path);
            if ui.button("Pick...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    state.open_project_path = path.display().to_string();
                }
            }
        });

        let can_open = !state.open_project_path.trim().is_empty();
        if ui
            .add_enabled(can_open, egui::Button::new("Open Project"))
            .clicked()
        {
            let root = PathBuf::from(state.open_project_path.trim());
            state.pending_commands.push(ProjectCommand::Open { root });
        }
    });

    ui.separator();
    ui.heading("Recent Projects");

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

fn sync_from_project(state: &mut ProjectPanelState, info: &editor_core::project::ProjectInfo) {
    let project_key = info.root.to_string_lossy().to_string();
    let world_key = info
        .current_world_id
        .clone()
        .unwrap_or_else(|| "none".to_string());
    if state.project_key.as_deref() == Some(&project_key)
        && state.world_key.as_deref() == Some(&world_key)
    {
        return;
    }

    state.project_key = Some(project_key);
    state.world_key = Some(world_key);
    state.project_name_edit = info.manifest.project_name.clone();

    if let Some(world) = info.current_world() {
        state.world_name_edit = world.manifest.world_name.clone();
        state.tile_size_meters = world.manifest.world_spec.tile_size_meters;
        state.chunks_per_tile = world.manifest.world_spec.chunks_per_tile;
        state.heightfield_samples = world.manifest.world_spec.heightfield_samples;
        state.weightmap_resolution = world.manifest.world_spec.weightmap_resolution;
        state.liquids_resolution = world.manifest.world_spec.liquids_resolution;
    }
}

fn world_spec_from_state(state: &ProjectPanelState) -> WorldSpec {
    WorldSpec {
        tile_size_meters: state.tile_size_meters,
        chunks_per_tile: state.chunks_per_tile,
        heightfield_samples: state.heightfield_samples,
        weightmap_resolution: state.weightmap_resolution,
        liquids_resolution: state.liquids_resolution,
    }
}

fn is_world_dirty(state: &ProjectPanelState, world: &editor_core::project::WorldInfo) -> bool {
    state.world_name_edit.trim() != world.manifest.world_name
        || state.tile_size_meters != world.manifest.world_spec.tile_size_meters
        || state.chunks_per_tile != world.manifest.world_spec.chunks_per_tile
        || state.heightfield_samples != world.manifest.world_spec.heightfield_samples
        || state.weightmap_resolution != world.manifest.world_spec.weightmap_resolution
        || state.liquids_resolution != world.manifest.world_spec.liquids_resolution
}

fn world_manifest_from_state(
    state: &ProjectPanelState,
    world: &editor_core::project::WorldInfo,
) -> world::schema::WorldManifest {
    let mut updated = world.manifest.clone();
    let world_name = state.world_name_edit.trim();
    updated.world_name = if world_name.is_empty() {
        world.manifest.world_name.clone()
    } else {
        world_name.to_string()
    };
    updated.world_spec = world_spec_from_state(state);
    updated
}

fn can_add_region(state: &ProjectPanelState, world: &editor_core::project::WorldInfo) -> bool {
    let region_id = state.new_region_id.trim();
    let region_name = state.new_region_name.trim();
    if region_id.is_empty() || region_name.is_empty() {
        return false;
    }
    let bounds = RegionBounds::new(
        state.region_min_x,
        state.region_min_y,
        state.region_max_x,
        state.region_max_y,
    );
    if !bounds.is_valid() {
        return false;
    }
    !world
        .manifest
        .regions
        .iter()
        .any(|region| region.region_id == region_id)
}

fn can_create_project(state: &ProjectPanelState) -> bool {
    if state.new_project_path.trim().is_empty() && state.new_project_name.trim().is_empty() {
        return false;
    }
    let bounds = RegionBounds::new(
        state.region_min_x,
        state.region_min_y,
        state.region_max_x,
        state.region_max_y,
    );
    if !bounds.is_valid() {
        return false;
    }
    !state.new_region_id.trim().is_empty() && !state.new_region_name.trim().is_empty()
}

fn resolve_project_root(state: &ProjectPanelState) -> PathBuf {
    let trimmed = state.new_project_path.trim();
    if !trimmed.is_empty() {
        return PathBuf::from(trimmed);
    }

    let base_dir = default_projects_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| ".".into()));
    let folder_name = sanitized_name(state.new_project_name.trim());
    let preferred = base_dir.join(folder_name);
    unique_project_root(preferred)
}

fn default_projects_dir() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        let home = std::env::var("USERPROFILE").ok()?;
        return Some(
            PathBuf::from(home)
                .join("Documents")
                .join("BevyTool")
                .join("Projects"),
        );
    }

    let home = std::env::var("HOME").ok()?;
    if cfg!(target_os = "macos") {
        return Some(
            PathBuf::from(home)
                .join("Documents")
                .join("BevyTool")
                .join("Projects"),
        );
    }

    Some(PathBuf::from(home).join("BevyTool").join("Projects"))
}

fn sanitized_name(name: &str) -> String {
    let trimmed = if name.is_empty() { "NewProject" } else { name };
    let sanitized = trimmed
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .to_string();

    if sanitized.is_empty() {
        "NewProject".to_string()
    } else {
        sanitized
    }
}

fn unique_project_root(preferred: PathBuf) -> PathBuf {
    if !preferred.exists() {
        return preferred;
    }
    let base = preferred
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("NewProject")
        .to_string();
    let parent = preferred.parent().map(Path::to_path_buf);
    for index in 1..100 {
        let candidate = format!("{base}-{index}");
        let Some(parent) = &parent else {
            break;
        };
        let path = parent.join(&candidate);
        if !path.exists() {
            return path;
        }
    }
    preferred
}
