use bevy_egui::egui;
use editor_core::project::{ActiveRegion, NewWorldRequest, ProjectCommand, ProjectInfo, WorldInfo};
use world::schema::{RegionBounds, RegionManifest};

use super::super::helpers::{
    can_add_region, can_create_world, is_world_dirty, sync_from_project, world_manifest_from_state,
    world_spec_from_new_world,
};
use super::super::ProjectPanelState;

pub(super) fn draw_active_project(
    ui: &mut egui::Ui,
    state: &mut ProjectPanelState,
    info: &ProjectInfo,
    active_region: &mut ActiveRegion,
) {
    sync_from_project(state, info);

    ui.label(format!("Root: {}", info.root.display()));
    ui.separator();

    draw_project_settings(ui, state, info);

    if let Some(world) = info.current_world() {
        draw_world_settings(ui, state, info, world);
        draw_add_world(ui, state, info);
        draw_regions(ui, state, info, world, active_region);
    }
}

fn draw_project_settings(ui: &mut egui::Ui, state: &mut ProjectPanelState, info: &ProjectInfo) {
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
}

fn draw_world_settings(
    ui: &mut egui::Ui,
    state: &mut ProjectPanelState,
    info: &ProjectInfo,
    world: &WorldInfo,
) {
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
                    .selectable_value(&mut selected_world, entry.manifest.world_id.clone(), label)
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
}

fn draw_regions(
    ui: &mut egui::Ui,
    state: &mut ProjectPanelState,
    info: &ProjectInfo,
    world: &WorldInfo,
    active_region: &mut ActiveRegion,
) {
    ui.separator();
    ui.heading("Regions");
    if world.manifest.regions.is_empty() {
        ui.label("No regions yet.");
    } else {
        let mut selected = active_region.region_id.clone();
        let selection_valid = selected
            .as_deref()
            .and_then(|id| {
                world
                    .manifest
                    .regions
                    .iter()
                    .find(|region| region.region_id == id)
            })
            .is_some();
        if !selection_valid {
            selected = Some(world.manifest.regions[0].region_id.clone());
            active_region.region_id = selected.clone();
        }

        ui.horizontal(|ui| {
            ui.label("Active region");
            egui::ComboBox::from_id_salt("active_region_select")
                .selected_text(
                    selected
                        .as_deref()
                        .unwrap_or(world.manifest.regions[0].region_id.as_str()),
                )
                .show_ui(ui, |ui| {
                    for region in &world.manifest.regions {
                        if ui
                            .selectable_value(
                                &mut selected,
                                Some(region.region_id.clone()),
                                format!("{} ({})", region.name, region.region_id),
                            )
                            .clicked()
                        {
                            active_region.region_id = selected.clone();
                        }
                    }
                });
        });
        ui.separator();
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

fn draw_add_world(ui: &mut egui::Ui, state: &mut ProjectPanelState, info: &ProjectInfo) {
    ui.separator();
    ui.heading("Add World");
    ui.horizontal(|ui| {
        ui.label("World name");
        ui.text_edit_singleline(&mut state.new_world.name);
    });
    ui.horizontal(|ui| {
        ui.label("Region id");
        ui.text_edit_singleline(&mut state.new_world.region_id);
    });
    ui.horizontal(|ui| {
        ui.label("Region name");
        ui.text_edit_singleline(&mut state.new_world.region_name);
    });
    ui.horizontal(|ui| {
        ui.label("Bounds min");
        ui.add(egui::DragValue::new(&mut state.new_world.region_min_x));
        ui.add(egui::DragValue::new(&mut state.new_world.region_min_y));
    });
    ui.horizontal(|ui| {
        ui.label("Bounds max");
        ui.add(egui::DragValue::new(&mut state.new_world.region_max_x));
        ui.add(egui::DragValue::new(&mut state.new_world.region_max_y));
    });

    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Tile size (m)");
        ui.add(
            egui::DragValue::new(&mut state.new_world.tile_size_meters)
                .speed(1.0)
                .range(1.0..=1_000_000.0),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Chunks per tile");
        ui.add(egui::DragValue::new(&mut state.new_world.chunks_per_tile).range(1..=64));
    });
    ui.horizontal(|ui| {
        ui.label("Heightfield samples");
        ui.add(egui::DragValue::new(&mut state.new_world.heightfield_samples).range(2..=4096));
    });
    ui.horizontal(|ui| {
        ui.label("Weightmap resolution");
        ui.add(egui::DragValue::new(&mut state.new_world.weightmap_resolution).range(2..=4096));
    });
    ui.horizontal(|ui| {
        ui.label("Liquids resolution");
        ui.add(egui::DragValue::new(&mut state.new_world.liquids_resolution).range(2..=4096));
    });

    let can_create = can_create_world(&state.new_world);
    if ui
        .add_enabled(can_create, egui::Button::new("Create World"))
        .clicked()
    {
        state.pending_commands.push(ProjectCommand::CreateWorld {
            root: info.root.clone(),
            request: NewWorldRequest {
                world_name: state.new_world.name.clone(),
                region_id: state.new_world.region_id.clone(),
                region_name: state.new_world.region_name.clone(),
                region_bounds: RegionBounds::new(
                    state.new_world.region_min_x,
                    state.new_world.region_min_y,
                    state.new_world.region_max_x,
                    state.new_world.region_max_y,
                ),
                world_spec: world_spec_from_new_world(&state.new_world),
            },
        });
    }
}
