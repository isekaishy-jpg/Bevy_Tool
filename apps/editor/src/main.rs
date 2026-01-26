use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use editor_core::log_capture::log_capture_layer;
use editor_core::EditorCorePlugin;
use editor_ui::EditorUiPlugin;
use viewport::ViewportPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            custom_layer: log_capture_layer,
            ..Default::default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(EditorCorePlugin)
        .add_plugins(EditorUiPlugin)
        .add_plugins(ViewportPlugin)
        .run();
}
