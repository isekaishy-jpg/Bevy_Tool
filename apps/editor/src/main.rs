use bevy::prelude::*;
use editor_core::EditorCorePlugin;
use editor_ui::EditorUiPlugin;
use viewport::ViewportPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EditorCorePlugin)
        .add_plugins(EditorUiPlugin)
        .add_plugins(ViewportPlugin)
        .run();
}
