use bevy::prelude::Resource;
use bevy_egui::egui;
use editor_core::log_capture::LogBuffer;
use editor_core::project::ProjectState;
use editor_core::EditorConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevelFilter {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevelFilter {
    fn all() -> [LogLevelFilter; 5] {
        [
            LogLevelFilter::Trace,
            LogLevelFilter::Debug,
            LogLevelFilter::Info,
            LogLevelFilter::Warn,
            LogLevelFilter::Error,
        ]
    }

    fn label(self) -> &'static str {
        match self {
            LogLevelFilter::Trace => "Trace",
            LogLevelFilter::Debug => "Debug",
            LogLevelFilter::Info => "Info",
            LogLevelFilter::Warn => "Warn",
            LogLevelFilter::Error => "Error",
        }
    }
}

#[derive(Resource)]
pub struct LogPanelState {
    pub search: String,
    pub min_level: LogLevelFilter,
    pub auto_scroll: bool,
    pub show_target: bool,
    pub wrap_lines: bool,
}

impl Default for LogPanelState {
    fn default() -> Self {
        Self {
            search: String::new(),
            min_level: LogLevelFilter::Info,
            auto_scroll: true,
            show_target: false,
            wrap_lines: false,
        }
    }
}

pub fn draw_log_panel(
    ui: &mut egui::Ui,
    state: &mut LogPanelState,
    log_buffer: Option<&LogBuffer>,
    project_state: &ProjectState,
    config: &EditorConfig,
) {
    ui.heading("Console");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Search");
        ui.text_edit_singleline(&mut state.search);
        ui.separator();
        ui.label("Level");
        egui::ComboBox::from_id_salt("log_level_filter")
            .selected_text(state.min_level.label())
            .show_ui(ui, |ui| {
                for level in LogLevelFilter::all() {
                    ui.selectable_value(&mut state.min_level, level, level.label());
                }
            });
    });

    ui.horizontal(|ui| {
        ui.checkbox(&mut state.auto_scroll, "Auto-scroll");
        ui.checkbox(&mut state.show_target, "Show target");
        ui.checkbox(&mut state.wrap_lines, "Wrap lines");
        if ui.button("Copy diagnostics").clicked() {
            let text = diagnostics_text(project_state, config, log_buffer);
            ui.ctx().copy_text(text);
        }
    });

    ui.separator();

    let Some(buffer) = log_buffer else {
        ui.label("Log capture is not initialized.");
        return;
    };

    let entries = buffer.snapshot();
    let query = state.search.trim().to_lowercase();
    let min_level = state.min_level;

    let mut scroll = egui::ScrollArea::vertical();
    if state.auto_scroll {
        scroll = scroll.stick_to_bottom(true);
    }

    scroll.show(ui, |ui| {
        ui.style_mut().wrap_mode = if state.wrap_lines {
            Some(egui::TextWrapMode::Wrap)
        } else {
            Some(egui::TextWrapMode::Extend)
        };

        for entry in entries {
            if !level_allowed(entry.level, min_level) {
                continue;
            }
            if !query.is_empty() && !matches_query(&entry, &query) {
                continue;
            }

            let timestamp = format_timestamp(entry.timestamp_unix_ms);
            let mut line = format!("{timestamp} [{}] {}", entry.level, entry.message);
            if state.show_target {
                line.push_str(&format!(" ({})", entry.target));
            }

            let color = level_color(entry.level);
            ui.colored_label(color, line);
        }
    });
}

fn matches_query(entry: &editor_core::log_capture::LogEntry, query: &str) -> bool {
    entry.message.to_lowercase().contains(query) || entry.target.to_lowercase().contains(query)
}

fn level_allowed(level: bevy::log::tracing::Level, min_level: LogLevelFilter) -> bool {
    use bevy::log::tracing::Level;

    let min = match min_level {
        LogLevelFilter::Trace => Level::TRACE,
        LogLevelFilter::Debug => Level::DEBUG,
        LogLevelFilter::Info => Level::INFO,
        LogLevelFilter::Warn => Level::WARN,
        LogLevelFilter::Error => Level::ERROR,
    };
    level >= min
}

fn level_color(level: bevy::log::tracing::Level) -> egui::Color32 {
    use bevy::log::tracing::Level;

    match level {
        Level::ERROR => egui::Color32::LIGHT_RED,
        Level::WARN => egui::Color32::YELLOW,
        Level::INFO => egui::Color32::LIGHT_GRAY,
        Level::DEBUG => egui::Color32::GRAY,
        Level::TRACE => egui::Color32::DARK_GRAY,
    }
}

fn format_timestamp(timestamp_ms: u64) -> String {
    let seconds = timestamp_ms / 1000;
    let millis = timestamp_ms % 1000;
    format!("{seconds}.{millis:03}")
}

fn diagnostics_text(
    project_state: &ProjectState,
    config: &EditorConfig,
    log_buffer: Option<&LogBuffer>,
) -> String {
    let mut lines = Vec::new();
    lines.push("Bevy Tool Diagnostics".to_string());
    lines.push(format!(
        "OS: {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    lines.push(format!("Project: {}", config.project_name));
    lines.push(format!("World: {}", config.world_name));
    if let Some(project) = &project_state.current {
        lines.push(format!("Project root: {}", project.root.display()));
        if let Some(world) = project.current_world() {
            lines.push(format!(
                "World id: {} ({})",
                world.manifest.world_id, world.manifest.world_name
            ));
        }
    } else {
        lines.push("Project root: <none>".to_string());
    }
    if let Some(error) = &project_state.last_error {
        lines.push(format!("Last error: {error}"));
    }

    if let Some(buffer) = log_buffer {
        let entries = buffer.snapshot();
        let tail = entries.len().saturating_sub(200);
        lines.push(String::new());
        lines.push("Log tail (last 200):".to_string());
        for entry in entries.into_iter().skip(tail) {
            lines.push(format!(
                "{} [{}] {} ({})",
                format_timestamp(entry.timestamp_unix_ms),
                entry.level,
                entry.message,
                entry.target
            ));
        }
    }

    lines.join("\n")
}
