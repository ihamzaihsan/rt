use crate::{
    geometry::{Hit, Object},
    math::{Ray, Vec3},
    scene::Scene,
};
use std::io::{self, Write};
const EPSILON: f64 = 1.0e-4;
pub(crate) struct RenderOptions {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) reflections: bool,
    pub(crate) max_depth: usize,
}

pub(crate) fn trace_ray(ray: Ray, scene: &Scene, options: &RenderOptions, depth: usize) -> Vec3 {
    let Some(hit) = closest_hit(ray, &scene.objects) else {
        return sky_color(ray.direction, scene.background);
    };

    let normal = if ray.direction.dot(hit.normal) < 0.0 {
        hit.normal
    } else {
        -hit.normal
    };
    let mut color = hit.material.color * scene.ambient;
    for light in &scene.lights {
        let to_light = light.position - hit.point;
        let distance_to_light = to_light.length();
        if distance_to_light < EPSILON {
            continue;
        }
        let light_dir = to_light / distance_to_light;
        let shadow_ray = Ray {
            origin: hit.point + normal * EPSILON,
            direction: light_dir,
        };

        let in_shadow = closest_hit(shadow_ray, &scene.objects)
            .is_some_and(|shadow_hit| shadow_hit.t < distance_to_light);
        if in_shadow {
            continue;
        }

        let diffuse = normal.dot(light_dir).max(0.0);
        let view_dir = -ray.direction;
        let reflect_dir = (-light_dir).reflect(normal).normalize();
        let specular = view_dir.dot(reflect_dir).max(0.0).powf(48.0) * 0.35;
        let attenuation = 1.0 / (1.0 + 0.025 * distance_to_light * distance_to_light);
        let light_strength = light.brightness * attenuation;

        color += hit.material.color * light.color * (diffuse * light_strength);
        color += light.color * (specular * light_strength);
    }

    if options.reflections && depth < options.max_depth && hit.material.reflectivity > 0.0 {
        let reflection_ray = Ray {
            origin: hit.point + normal * EPSILON,
            direction: ray.direction.reflect(normal).normalize(),
        };
        let reflected = trace_ray(reflection_ray, scene, options, depth + 1);
        color = color * (1.0 - hit.material.reflectivity) + reflected * hit.material.reflectivity;
    }

    color.clamp01()
}

pub(crate) fn closest_hit(ray: Ray, objects: &[Object]) -> Option<Hit> {
    objects
        .iter()
        .filter_map(|object| object.intersect(ray))
        .min_by(|a, b| a.t.total_cmp(&b.t))
}

pub(crate) fn sky_color(direction: Vec3, base: Vec3) -> Vec3 {
    let t = 0.5 * (direction.y + 1.0);
    base * (1.0 - t) + Vec3::new(0.68, 0.78, 0.95) * t
}

pub(crate) fn render<W: Write>(
    scene: &Scene,
    options: &RenderOptions,
    mut writer: W,
) -> io::Result<()> {
    writeln!(writer, "P3")?;
    writeln!(writer, "{} {}", options.width, options.height)?;
    writeln!(writer, "255")?;

    for y in 0..options.height {
        for x in 0..options.width {
            let ray = scene
                .camera
                .ray_for_pixel(x, y, options.width, options.height);
            let color = trace_ray(ray, scene, options, 0).clamp01();
            let r = (color.x * 255.0).round() as u8;
            let g = (color.y * 255.0).round() as u8;
            let b = (color.z * 255.0).round() as u8;
            writeln!(writer, "{r} {g} {b}")?;
        }
    }

    writer.flush()
}
