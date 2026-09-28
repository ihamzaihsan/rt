use crate::{
    camera::Camera,
    geometry::{Checker, Material, Object},
    math::Vec3,
};

#[derive(Clone, Copy)]
pub(crate) struct Light {
    pub(crate) position: Vec3,
    pub(crate) color: Vec3,
    pub(crate) brightness: f64,
}

pub(crate) struct Scene {
    pub(crate) objects: Vec<Object>,
    pub(crate) lights: Vec<Light>,
    pub(crate) camera: Camera,
    pub(crate) background: Vec3,
    pub(crate) ambient: f64,
}

pub(crate) fn material(color: Vec3, reflectivity: f64) -> Material {
    Material {
        color,
        reflectivity,
        transmission: 0.0,
        ior: 1.5,
        texture: Some(Checker {
            alternate: color * 0.3,
            scale: 2.0,
        }),
    }
}

pub(crate) fn scene_by_name(
    name: &str,
    brightness: Option<f64>,
    camera_override: Option<Vec3>,
    target_override: Option<Vec3>,
    fov_override: Option<f64>,
) -> Result<Scene, String> {
    match name {
        "sphere" => {
            let camera_pos = camera_override.unwrap_or(Vec3::new(0.0, 1.0, 4.5));
            Ok(Scene {
                objects: vec![
                    Object::Sphere {
                        center: Vec3::new(0.0, 1.0, 0.0),
                        radius: 1.0,
                        material: material(Vec3::new(0.95, 0.18, 0.14), 0.35),
                    },
                    Object::Plane {
                        point: Vec3::new(0.0, 0.0, 0.0),
                        normal: Vec3::new(0.0, 1.0, 0.0),
                        material: material(Vec3::new(0.72, 0.74, 0.70), 0.12),
                    },
                ],
                lights: vec![Light {
                    position: Vec3::new(-3.0, 5.5, 3.0),
                    color: Vec3::new(1.0, 0.96, 0.88),
                    brightness: brightness.unwrap_or(5.5),
                }],
                camera: Camera::look_at(
                    camera_pos,
                    target_override.unwrap_or(Vec3::new(0.0, 0.8, 0.0)),
                    Vec3::new(0.0, 1.0, 0.0),
                    fov_override.unwrap_or(50.0),
                )?,
                background: Vec3::new(0.08, 0.10, 0.14),
                ambient: 0.12,
            })
        }
        "cube-plane" => {
            let camera_pos = camera_override.unwrap_or(Vec3::new(2.4, 1.8, 5.2));
            Ok(Scene {
                objects: vec![
                    Object::Plane {
                        point: Vec3::new(0.0, -0.55, 0.0),
                        normal: Vec3::new(0.0, 1.0, 0.0),
                        material: material(Vec3::new(0.62, 0.67, 0.58), 0.08),
                    },
                    Object::Cube {
                        min: Vec3::new(-0.8, -0.55, -0.8),
                        max: Vec3::new(0.8, 1.05, 0.8),
                        material: material(Vec3::new(0.15, 0.43, 0.82), 0.18),
                    },
                ],
                lights: vec![Light {
                    position: Vec3::new(-2.2, 4.2, 3.6),
                    color: Vec3::new(1.0, 0.95, 0.86),
                    brightness: brightness.unwrap_or(3.0),
                }],
                camera: Camera::look_at(
                    camera_pos,
                    target_override.unwrap_or(Vec3::new(0.0, 0.25, 0.0)),
                    Vec3::new(0.0, 1.0, 0.0),
                    fov_override.unwrap_or(48.0),
                )?,
                background: Vec3::new(0.07, 0.08, 0.10),
                ambient: 0.10,
            })
        }
        "all" | "all-alt" => {
            let default_camera = if name == "all-alt" {
                Vec3::new(-4.4, 2.3, 3.2)
            } else {
                Vec3::new(3.7, 2.4, 5.0)
            };
            let camera_pos = camera_override.unwrap_or(default_camera);
            Ok(Scene {
                objects: vec![
                    Object::Plane {
                        point: Vec3::new(0.0, -0.75, 0.0),
                        normal: Vec3::new(0.0, 1.0, 0.0),
                        material: material(Vec3::new(0.66, 0.69, 0.63), 0.10),
                    },
                    Object::Sphere {
                        center: Vec3::new(-1.35, 0.05, 0.15),
                        radius: 0.8,
                        material: material(Vec3::new(0.94, 0.25, 0.18), 0.32),
                    },
                    Object::Cube {
                        min: Vec3::new(0.35, -0.75, -0.55),
                        max: Vec3::new(1.55, 0.45, 0.65),
                        material: material(Vec3::new(0.16, 0.45, 0.78), 0.20),
                    },
                    Object::Cylinder {
                        center: Vec3::new(0.0, 0.05, -1.55),
                        radius: 0.45,
                        height: 1.6,
                        material: material(Vec3::new(0.91, 0.68, 0.20), 0.24),
                    },
                ],
                lights: vec![
                    Light {
                        position: Vec3::new(-2.8, 5.0, 3.3),
                        color: Vec3::new(1.0, 0.95, 0.86),
                        brightness: brightness.unwrap_or(5.0),
                    },
                    Light {
                        position: Vec3::new(3.8, 3.2, -2.6),
                        color: Vec3::new(0.62, 0.72, 1.0),
                        brightness: brightness.unwrap_or(5.0) * 0.65,
                    },
                ],
                camera: Camera::look_at(
                    camera_pos,
                    target_override.unwrap_or(Vec3::new(0.0, 0.0, -0.35)),
                    Vec3::new(0.0, 1.0, 0.0),
                    fov_override.unwrap_or(50.0),
                )?,
                background: Vec3::new(0.08, 0.09, 0.12),
                ambient: 0.11,
            })
        }
        _ => Err(format!(
            "unknown scene '{name}'. Use --help for valid scenes."
        )),
    }
}

/// Attach opt-in effects without changing the four original scene layouts.
pub(crate) fn add_effects(
    scene: &mut Scene,
    refractive: bool,
    particle_effect: bool,
    fluid: bool,
    time: f64,
) {
    if refractive {
        for object in &mut scene.objects {
            if let Object::Sphere { material, .. } = object {
                material.transmission = 0.92;
                material.color = Vec3::new(0.94, 0.98, 1.0);
                material.texture = None;
            }
        }
    }
    let floor = scene
        .objects
        .iter()
        .find_map(|object| {
            if let Object::Plane { point, normal, .. } = object {
                if normal.x.abs() < 1.0e-12 && normal.z.abs() < 1.0e-12 && normal.y.abs() > 1.0e-12
                {
                    return Some(point.y);
                }
            }
            None
        })
        .unwrap_or(0.0);
    if particle_effect {
        let mut droplet = material(Vec3::new(0.12, 0.65, 0.92), 0.3);
        droplet.texture = None;
        scene.objects.extend(crate::effects::particles(
            time,
            Vec3::new(0.0, floor + 0.3, 0.6),
            droplet,
        ));
    }
    if fluid {
        let mut water = material(Vec3::new(0.05, 0.32, 0.48), 0.55);
        water.texture = None;
        water.ior = 1.333;
        water.transmission = if refractive { 0.7 } else { 0.0 };
        scene.objects.push(Object::Fluid {
            surface: crate::effects::FluidSurface {
                min: Vec3::new(-3.0, 0.0, -3.0),
                max: Vec3::new(3.0, 0.0, 2.5),
                level: floor + 0.22,
                amplitude: 0.07,
                time,
            },
            material: water,
        });
    }
}
