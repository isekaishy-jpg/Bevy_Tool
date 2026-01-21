use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::log::info;
use bevy::prelude::{KeyCode, Query, Res, ResMut, Resource, With};
use bevy::window::{PrimaryWindow, Window};

use crate::ViewportRect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportCaptureSource {
    Default,
    Tool,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportCaptureRequest {
    pub source: ViewportCaptureSource,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportCaptureChanged {
    pub captured: bool,
    pub source: Option<ViewportCaptureSource>,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ViewportUiInput {
    pub wants_pointer: bool,
    pub wants_keyboard: bool,
}
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ViewportInputState {
    pub hovered: bool,
    pub focused: bool,
    pub captured: bool,
    pub captor: Option<ViewportCaptureSource>,
    pub hotkeys_allowed: bool,
}

#[derive(SystemParam)]
pub struct ViewportInputParams<'w, 's> {
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    capture_requests: MessageReader<'w, 's, ViewportCaptureRequest>,
    capture_changed: MessageWriter<'w, ViewportCaptureChanged>,
}

pub fn update_viewport_input(
    rect: Res<ViewportRect>,
    ui_input: Res<ViewportUiInput>,
    mut input: ViewportInputParams,
    mut state: ResMut<ViewportInputState>,
) {
    let mut hovered = false;
    if rect.is_valid {
        if let Ok(window) = input.windows.single() {
            if let Some(cursor) = window.cursor_position() {
                hovered = rect.logical_rect().contains(cursor);
            }
        }
    }

    let focusable = hovered;
    let mut next_captured = state.captured;
    let mut next_captor = state.captor;

    let mut requested_source = None;
    for request in input.capture_requests.read() {
        requested_source = Some(request.source);
    }

    if next_captured {
        if input.keys.just_pressed(KeyCode::Escape)
            || input.mouse_buttons.just_released(MouseButton::Left)
        {
            next_captured = false;
            next_captor = None;
        }
    } else if focusable && !ui_input.wants_pointer {
        if let Some(source) = requested_source {
            next_captured = true;
            next_captor = Some(source);
        } else if input.mouse_buttons.just_pressed(MouseButton::Left) {
            next_captured = true;
            next_captor = Some(ViewportCaptureSource::Default);
        }
    }

    let focused = next_captured || focusable;
    let hotkeys_allowed = focused && !ui_input.wants_keyboard;
    let changed = next_captured != state.captured || next_captor != state.captor;

    state.hovered = hovered;
    state.focused = focused;
    state.captured = next_captured;
    state.captor = next_captor;
    state.hotkeys_allowed = hotkeys_allowed;

    if changed {
        input.capture_changed.write(ViewportCaptureChanged {
            captured: next_captured,
            source: next_captor,
        });
    }
}

pub fn log_viewport_capture_changes(mut messages: MessageReader<ViewportCaptureChanged>) {
    for message in messages.read() {
        info!(
            "viewport capture {} (source: {:?})",
            if message.captured {
                "enabled"
            } else {
                "disabled"
            },
            message.source
        );
    }
}
