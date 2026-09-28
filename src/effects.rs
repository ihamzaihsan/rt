//! Deterministic particle motion and a procedural fluid height field.
use crate::{
    geometry::{Hit, Material, Object},
    math::{Ray, Vec3},
};

#[derive(Clone, Copy)]
pub(crate) struct FluidSurface {
    pub(crate) min: Vec3,
    pub(crate) max: Vec3,
    pub(crate) level: f64,
    pub(crate) amplitude: f64,
    pub(crate) time: f64,
}

impl FluidSurface {
    pub(crate) fn height(self, x: f64, z: f64) -> f64 {
        self.level
            + self.amplitude
                * ((2.0 * x + self.time).sin() * (1.7 * z - 0.8 * self.time).cos()
                    + 0.35 * (3.5 * z + 0.6 * self.time).sin())
    }

    pub(crate) fn normal(self, x: f64, z: f64) -> Vec3 {
        let dx =
            2.0 * self.amplitude * (2.0 * x + self.time).cos() * (1.7 * z - 0.8 * self.time).cos();
        let dz = self.amplitude
            * (-1.7 * (2.0 * x + self.time).sin() * (1.7 * z - 0.8 * self.time).sin()
                + 1.225 * (3.5 * z + 0.6 * self.time).cos());
        Vec3::new(-dx, 1.0, -dz).normalize()
    }

    pub(crate) fn intersect(self, ray: Ray, material: Material) -> Option<Hit> {
        // Clip to the wave's enclosing box before evaluating the height field.
        let mut near: f64 = 1.0e-4;
        let mut far = f64::INFINITY;
        for (origin, direction, min, max) in [
            (ray.origin.x, ray.direction.x, self.min.x, self.max.x),
            (
                ray.origin.y,
                ray.direction.y,
                self.level - 1.35 * self.amplitude,
                self.level + 1.35 * self.amplitude,
            ),
            (ray.origin.z, ray.direction.z, self.min.z, self.max.z),
        ] {
            if direction.abs() < 1.0e-12 {
                if origin < min || origin > max {
                    return None;
                }
            } else {
                let a = (min - origin) / direction;
                let b = (max - origin) / direction;
                near = near.max(a.min(b));
                far = far.min(a.max(b));
            }
        }
        if near > far {
            return None;
        }

        // A bound on |d(y - height(x,z))/dt| gives a conservative step, so
        // steep waves are not skipped as they can be with fixed-step marching.
        let slope_bound = ray.direction.y.abs()
            + self.amplitude * (2.0 * ray.direction.x.abs() + 2.925 * ray.direction.z.abs());
        if slope_bound < 1.0e-12 {
            return None;
        }
        let mut t = near;
        for _ in 0..256 {
            let point = ray.at(t);
            let distance = point.y - self.height(point.x, point.z);
            if distance.abs() < 1.0e-6 {
                return Some(Hit {
                    t,
                    point,
                    normal: self.normal(point.x, point.z),
                    material,
                });
            }
            t += 0.9 * distance.abs() / slope_bound;
            if t > far {
                return None;
            }
        }
        None
    }
}

/// A repeating fountain of small spheres, with analytical ballistic trajectories.
pub(crate) fn particles(time: f64, origin: Vec3, material: Material) -> Vec<Object> {
    (0..32)
        .map(|index| {
            let phase = index as f64 / 32.0;
            let age = (time + phase * 1.4).rem_euclid(1.4);
            let angle = index as f64 * 2.399_963_229_728_653; // Golden angle.
            let speed = 0.5 + 0.5 * phase;
            let velocity = Vec3::new(angle.cos() * speed, 3.8 + phase * 0.6, angle.sin() * speed);
            let center = origin + velocity * age + Vec3::new(0.0, -2.8 * age * age, 0.0);
            Object::Sphere {
                center,
                radius: 0.035 + phase * 0.025,
                material,
            }
        })
        .collect()
}
