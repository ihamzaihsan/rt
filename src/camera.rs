use crate::math::{Ray, Vec3};
pub(crate) struct Camera {
    pub(crate) origin: Vec3,
    pub(crate) forward: Vec3,
    pub(crate) right: Vec3,
    pub(crate) up: Vec3,
    pub(crate) fov_radians: f64,
}

impl Camera {
    pub(crate) fn look_at(origin: Vec3, target: Vec3, up_hint: Vec3, fov_degrees: f64) -> Self {
        let forward = (target - origin).normalize();
        let right = forward.cross(up_hint).normalize();
        let up = right.cross(forward).normalize();
        Self {
            origin,
            forward,
            right,
            up,
            fov_radians: fov_degrees.to_radians(),
        }
    }

    pub(crate) fn ray_for_pixel(&self, x: usize, y: usize, width: usize, height: usize) -> Ray {
        let aspect_ratio = width as f64 / height as f64;
        let scale = (self.fov_radians / 2.0).tan();
        let px = (2.0 * ((x as f64 + 0.5) / width as f64) - 1.0) * aspect_ratio * scale;
        let py = (1.0 - 2.0 * ((y as f64 + 0.5) / height as f64)) * scale;
        Ray {
            origin: self.origin,
            direction: (self.forward + self.right * px + self.up * py).normalize(),
        }
    }
}
