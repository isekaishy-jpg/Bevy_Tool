use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use bevy::log::warn;
use bevy::prelude::{DetectChanges, Res, Resource};
use serde::{Deserialize, Serialize};

const PREFS_VERSION: u32 = 1;
const MAX_RECENT_PROJECTS: usize = 12;
const APP_NAME: &str = "BevyTool";

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
#[serde(default)]
pub struct EditorPrefs {
    pub prefs_version: u32,
    #[serde(alias = "last_project")]
    pub last_project_path: Option<String>,
    pub recent_projects: Vec<RecentProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: String,
    pub display_name: String,
    pub last_opened_unix_ms: u64,
}

impl Default for EditorPrefs {
    fn default() -> Self {
        Self {
            prefs_version: PREFS_VERSION,
            last_project_path: None,
            recent_projects: Vec::new(),
        }
    }
}

impl EditorPrefs {
    pub fn record_project(&mut self, path: &Path, display_name: String) {
        let normalized = normalize_path(path);
        self.last_project_path = Some(normalized.clone());

        if let Some(entry) = self
            .recent_projects
            .iter_mut()
            .find(|entry| entry.path == normalized)
        {
            entry.display_name = display_name;
            entry.last_opened_unix_ms = now_unix_ms();
        } else {
            self.recent_projects.push(RecentProject {
                path: normalized,
                display_name,
                last_opened_unix_ms: now_unix_ms(),
            });
        }

        self.recent_projects
            .sort_by_key(|entry| std::cmp::Reverse(entry.last_opened_unix_ms));
        if self.recent_projects.len() > MAX_RECENT_PROJECTS {
            self.recent_projects.truncate(MAX_RECENT_PROJECTS);
        }
    }

    pub fn remove_recent(&mut self, path: &Path) {
        let normalized = normalize_path(path);
        self.recent_projects
            .retain(|entry| entry.path != normalized);
        if self.last_project_path.as_deref() == Some(&normalized) {
            self.last_project_path = None;
        }
    }

    pub fn update_display_name(&mut self, path: &Path, display_name: &str) {
        let normalized = normalize_path(path);
        if let Some(entry) = self
            .recent_projects
            .iter_mut()
            .find(|entry| entry.path == normalized)
        {
            entry.display_name = display_name.to_string();
        }
    }
}

pub fn load_editor_prefs() -> anyhow::Result<EditorPrefs> {
    let path = prefs_path()?;
    if !path.exists() {
        return Ok(EditorPrefs::default());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("read prefs {:?}", path))?;
    let prefs = toml::from_str(&text)?;
    Ok(prefs)
}

pub fn save_editor_prefs(prefs: &EditorPrefs) -> anyhow::Result<()> {
    let path = prefs_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create prefs dir {:?}", parent))?;
    }
    let text = toml::to_string_pretty(prefs)?;
    fs::write(&path, text).with_context(|| format!("write prefs {:?}", path))?;
    Ok(())
}

pub fn save_prefs_on_change(prefs: Res<EditorPrefs>) {
    if prefs.is_changed() {
        if let Err(err) = save_editor_prefs(prefs.as_ref()) {
            warn!("failed to save editor prefs: {err}");
        }
    }
}

fn prefs_path() -> anyhow::Result<PathBuf> {
    let dir = prefs_dir()?;
    Ok(dir.join("editor.toml"))
}

fn prefs_dir() -> anyhow::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let app_data = env::var("APPDATA").context("APPDATA not set")?;
        Ok(PathBuf::from(app_data).join(APP_NAME))
    } else if cfg!(target_os = "macos") {
        let home = env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join(APP_NAME))
    } else if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        Ok(PathBuf::from(xdg).join(APP_NAME))
    } else {
        let home = env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".config").join(APP_NAME))
    }
}

fn normalize_path(path: &Path) -> String {
    match fs::canonicalize(path) {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(_) => path.to_string_lossy().to_string(),
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
