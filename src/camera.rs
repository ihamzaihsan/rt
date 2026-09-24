use crate::math::{Ray, Vec3};

pub(crate) struct Camera {
    origin: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    fov_radians: f64,
}

impl Camera {
    pub(crate) fn look_at(
        origin: Vec3,
        target: Vec3,
        up_hint: Vec3,
        fov_degrees: f64,
    ) -> Result<Self, String> {
        if !origin.is_finite() || !target.is_finite() || !up_hint.is_finite() {
            return Err("camera coordinates must be finite".into());
        }
        if !fov_degrees.is_finite() || !(1.0..179.0).contains(&fov_degrees) {
            return Err("field of view must be at least 1 and less than 179 degrees".into());
        }
        let distance = (target - origin).length();
        if !distance.is_finite() || distance < 1.0e-8 {
            return Err(
                "camera position and target must define a finite, nonzero direction".into(),
            );
        }
        let forward = (target - origin).normalize();
        // Avoid a degenerate basis when looking straight up or down.
        let up_hint = if forward.cross(up_hint).length() < 1.0e-8 {
            if forward.y.abs() < 0.9 {
                Vec3::new(0.0, 1.0, 0.0)
            } else {
                Vec3::new(1.0, 0.0, 0.0)
            }
        } else {
            up_hint
        };
        let right = forward.cross(up_hint).normalize();
        let up = right.cross(forward).normalize();
        Ok(Self {
            origin,
            forward,
            right,
            up,
            fov_radians: fov_degrees.to_radians(),
        })
    }

    pub(crate) fn ray_for_sample(&self, x: f64, y: f64, width: usize, height: usize) -> Ray {
        let aspect_ratio = width as f64 / height as f64;
        let scale = (self.fov_radians / 2.0).tan();
        let px = (2.0 * (x / width as f64) - 1.0) * aspect_ratio * scale;
        let py = (1.0 - 2.0 * (y / height as f64)) * scale;
        Ray {
            origin: self.origin,
            direction: (self.forward + self.right * px + self.up * py).normalize(),
        }
    }
}
