use bevy::prelude::{Rect, Resource, UVec2, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportBackend {
    CameraViewport,
    RenderToTexture,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct ViewportRect {
    pub logical_origin: Vec2,
    pub logical_size: Vec2,
    pub physical_origin: UVec2,
    pub physical_size: UVec2,
    pub scale_factor: f32,
    pub is_valid: bool,
}

impl Default for ViewportRect {
    fn default() -> Self {
        Self {
            logical_origin: Vec2::ZERO,
            logical_size: Vec2::ZERO,
            physical_origin: UVec2::ZERO,
            physical_size: UVec2::ZERO,
            scale_factor: 1.0,
            is_valid: false,
        }
    }
}

impl ViewportRect {
    pub fn invalidate(&mut self) {
        self.logical_origin = Vec2::ZERO;
        self.logical_size = Vec2::ZERO;
        self.physical_origin = UVec2::ZERO;
        self.physical_size = UVec2::ZERO;
        self.is_valid = false;
    }

    pub fn logical_rect(&self) -> Rect {
        Rect::from_corners(self.logical_origin, self.logical_origin + self.logical_size)
    }

    pub fn from_logical_rect(logical: Rect, screen: Rect, scale_factor: f32) -> Self {
        let clamped = logical.intersect(screen);
        let logical_size = clamped.size().max(Vec2::ZERO);
        let logical_origin = clamped.min;
        if !scale_factor.is_finite() || scale_factor <= 0.0 {
            return Self {
                logical_origin,
                logical_size,
                scale_factor,
                ..Default::default()
            };
        }

        let physical_origin = (logical_origin * scale_factor).floor();
        let physical_size = (logical_size * scale_factor).ceil();
        let physical_origin = UVec2::new(
            physical_origin.x.max(0.0) as u32,
            physical_origin.y.max(0.0) as u32,
        );
        let physical_size = UVec2::new(
            physical_size.x.max(0.0) as u32,
            physical_size.y.max(0.0) as u32,
        );

        let is_valid = logical_size.x > 0.0
            && logical_size.y > 0.0
            && physical_size.x > 0
            && physical_size.y > 0;

        Self {
            logical_origin,
            logical_size,
            physical_origin,
            physical_size,
            scale_factor,
            is_valid,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct ViewportService {
    pub backend: ViewportBackend,
    pub rect: ViewportRect,
}

impl Default for ViewportService {
    fn default() -> Self {
        Self {
            backend: ViewportBackend::CameraViewport,
            rect: ViewportRect::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_rect_clamps_to_screen() {
        let logical = Rect::from_corners(Vec2::new(-10.0, -5.0), Vec2::new(50.0, 40.0));
        let screen = Rect::from_corners(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let rect = ViewportRect::from_logical_rect(logical, screen, 2.0);

        assert_eq!(rect.logical_origin, Vec2::ZERO);
        assert_eq!(rect.logical_size, Vec2::new(50.0, 40.0));
        assert_eq!(rect.physical_origin, UVec2::ZERO);
        assert_eq!(rect.physical_size, UVec2::new(100, 80));
        assert!(rect.is_valid);
    }
}
