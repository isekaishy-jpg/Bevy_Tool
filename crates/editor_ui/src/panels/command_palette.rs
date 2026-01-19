use bevy::prelude::*;
use bevy_egui::egui;
use editor_core::command_registry::{CommandId, CommandInvoked, CommandRegistry};

#[derive(Resource, Debug, Default)]
pub struct CommandPaletteState {
    pub open: bool,
    pub just_opened: bool,
    pub query: String,
    pub selected: usize,
}

pub fn handle_command_palette_shortcuts(ctx: &egui::Context, state: &mut CommandPaletteState) {
    if ctx.input(|input| input.key_pressed(egui::Key::P) && input.modifiers.ctrl) && !state.open {
        state.open = true;
        state.just_opened = true;
        state.selected = 0;
    }
}

pub fn draw_command_palette(
    ctx: &egui::Context,
    state: &mut CommandPaletteState,
    registry: &CommandRegistry,
    commands: &mut Commands,
) {
    if !state.open {
        return;
    }

    let mut executed: Option<CommandId> = None;
    let mut close = false;

    egui::Window::new("Command Palette")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0.0, 32.0))
        .show(ctx, |ui| {
            let response = ui.text_edit_singleline(&mut state.query);
            if state.just_opened {
                response.request_focus();
                state.just_opened = false;
            }
            if response.changed() {
                state.selected = 0;
            }

            let query = state.query.trim().to_lowercase();
            let matches: Vec<_> = registry
                .commands
                .iter()
                .filter(|cmd| {
                    if query.is_empty() {
                        true
                    } else {
                        cmd.name.to_lowercase().contains(&query)
                            || cmd.description.to_lowercase().contains(&query)
                    }
                })
                .collect();

            if matches.is_empty() {
                ui.label("No matching commands.");
                return;
            }

            if state.selected >= matches.len() {
                state.selected = matches.len().saturating_sub(1);
            }

            let up = ui.input(|input| input.key_pressed(egui::Key::ArrowUp));
            let down = ui.input(|input| input.key_pressed(egui::Key::ArrowDown));
            let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
            let escape = ui.input(|input| input.key_pressed(egui::Key::Escape));

            if up && state.selected > 0 {
                state.selected -= 1;
            }
            if down && state.selected + 1 < matches.len() {
                state.selected += 1;
            }

            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(240.0)
                .show(ui, |ui| {
                    for (index, command) in matches.iter().enumerate() {
                        let label = format!(
                            "{}\n{}",
                            command.name,
                            if command.description.is_empty() {
                                command.id.as_str()
                            } else {
                                command.description
                            }
                        );
                        let response = ui.selectable_label(index == state.selected, label);
                        if response.clicked() {
                            executed = Some(command.id);
                        }
                    }
                });

            if enter {
                executed = Some(matches[state.selected].id);
            }
            if escape {
                close = true;
            }
        });

    if let Some(id) = executed {
        commands.trigger(CommandInvoked { id });
        close = true;
    }

    if close {
        state.open = false;
        state.query.clear();
        state.selected = 0;
    }
}
