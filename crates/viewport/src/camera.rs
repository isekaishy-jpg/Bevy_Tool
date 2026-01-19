//! Camera control policy (v0). Implement RTS/orbit controls in Bevy later.

#[derive(Debug, Clone, Copy)]
pub struct CameraTuning {
    pub base_speed: f32,
    pub speed_altitude_scale: f32,
}

impl Default for CameraTuning {
    fn default() -> Self {
        Self {
            base_speed: 10.0,
            speed_altitude_scale: 0.02,
        }
    }
}
