//! Editor tool state (placeholder until full tool system lands).

use bevy::prelude::Resource;

/// Active tool label shown in HUD readouts.
#[derive(Resource, Debug, Clone)]
pub struct ActiveTool {
    pub label: String,
}

impl Default for ActiveTool {
    fn default() -> Self {
        Self {
            label: "None".to_string(),
        }
    }
}
