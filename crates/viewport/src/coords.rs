use bevy::prelude::{Camera, Dir3, GlobalTransform, Ray3d, Vec2};

use crate::{ViewportBackend, ViewportService};

impl ViewportService {
    pub fn screen_to_viewport_local_physical(&self, cursor_logical: Vec2) -> Option<Vec2> {
        if self.backend != ViewportBackend::CameraViewport || !self.rect.is_valid {
            return None;
        }
        if !cursor_logical.is_finite() {
            return None;
        }
        if !self.rect.logical_rect().contains(cursor_logical) {
            return None;
        }
        let scale_factor = self.rect.scale_factor;
        if !scale_factor.is_finite() || scale_factor <= 0.0 {
            return None;
        }
        let cursor_physical = cursor_logical * scale_factor;
        let origin = self.rect.physical_origin.as_vec2();
        Some(cursor_physical - origin)
    }

    pub fn viewport_local_to_ndc(&self, local_physical: Vec2) -> Option<Vec2> {
        if self.backend != ViewportBackend::CameraViewport || !self.rect.is_valid {
            return None;
        }
        if !local_physical.is_finite() {
            return None;
        }
        let size = self.rect.physical_size.as_vec2();
        if size.x <= 0.0 || size.y <= 0.0 || !size.is_finite() {
            return None;
        }
        let mut ndc = (local_physical / size) * 2.0 - Vec2::ONE;
        ndc.y = -ndc.y;
        Some(ndc)
    }

    pub fn viewport_ray(
        &self,
        cursor_logical: Vec2,
        camera: &Camera,
        camera_transform: &GlobalTransform,
    ) -> Option<Ray3d> {
        if self.backend != ViewportBackend::CameraViewport {
            return None;
        }
        let local_physical = self.screen_to_viewport_local_physical(cursor_logical)?;
        let ndc = self.viewport_local_to_ndc(local_physical)?;
        let near = camera.ndc_to_world(camera_transform, ndc.extend(1.0))?;
        let far = camera.ndc_to_world(camera_transform, ndc.extend(f32::EPSILON))?;
        let direction = Dir3::new(far - near).ok()?;
        Some(Ray3d::new(near, direction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{Rect, Vec2};

    use crate::{ViewportRect, ViewportService};

    fn service_with_rect(rect: ViewportRect) -> ViewportService {
        ViewportService {
            backend: ViewportBackend::CameraViewport,
            rect,
        }
    }

    fn assert_vec2_approx(a: Vec2, b: Vec2) {
        let diff = a - b;
        assert!(diff.length() < 0.0001, "expected {b:?}, got {a:?}");
    }

    #[test]
    fn screen_to_local_physical_maps_with_scale() {
        let logical = Rect::from_corners(Vec2::new(100.0, 50.0), Vec2::new(300.0, 250.0));
        let screen = Rect::from_corners(Vec2::ZERO, Vec2::new(400.0, 300.0));
        let rect = ViewportRect::from_logical_rect(logical, screen, 2.0);
        let service = service_with_rect(rect);

        let local = service
            .screen_to_viewport_local_physical(Vec2::new(150.0, 100.0))
            .unwrap();
        assert_eq!(local, Vec2::new(100.0, 100.0));
        assert!(service
            .screen_to_viewport_local_physical(Vec2::new(10.0, 10.0))
            .is_none());
    }

    #[test]
    fn viewport_local_to_ndc_maps_corners() {
        let logical = Rect::from_corners(Vec2::ZERO, Vec2::new(200.0, 100.0));
        let rect = ViewportRect::from_logical_rect(logical, logical, 1.0);
        let service = service_with_rect(rect);

        let ndc_min = service.viewport_local_to_ndc(Vec2::ZERO).unwrap();
        assert_vec2_approx(ndc_min, Vec2::new(-1.0, 1.0));

        let ndc_max = service
            .viewport_local_to_ndc(Vec2::new(200.0, 100.0))
            .unwrap();
        assert_vec2_approx(ndc_max, Vec2::new(1.0, -1.0));
    }

    #[test]
    fn screen_to_local_physical_handles_fractional_scale() {
        let logical = Rect::from_corners(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let rect = ViewportRect::from_logical_rect(logical, logical, 1.5);
        let service = service_with_rect(rect);

        let local = service
            .screen_to_viewport_local_physical(Vec2::new(10.0, 20.0))
            .unwrap();
        assert_eq!(local, Vec2::new(15.0, 30.0));
    }
}
