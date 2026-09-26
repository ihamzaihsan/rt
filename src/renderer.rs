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
    pub(crate) refractions: bool,
    pub(crate) textures: bool,
    pub(crate) max_depth: usize,
}

fn trace_ray(ray: Ray, scene: &Scene, options: &RenderOptions, depth: usize) -> Vec3 {
    let Some(hit) = closest_hit(ray, &scene.objects) else {
        return sky_color(ray.direction, scene.background);
    };
    let entering = ray.direction.dot(hit.normal) < 0.0;
    let normal = if entering { hit.normal } else { -hit.normal };
    let surface_color = hit.material.color_at(hit.point, options.textures);
    let mut color = surface_color * scene.ambient;
    for light in &scene.lights {
        let to_light = light.position - hit.point;
        let distance_to_light = to_light.length();
        if distance_to_light < EPSILON {
            continue;
        }
        let light_dir = to_light / distance_to_light;
        let diffuse = normal.dot(light_dir).max(0.0);
        if diffuse == 0.0 {
            continue;
        }
        let shadow_ray = Ray {
            origin: hit.point + normal * EPSILON,
            direction: light_dir,
        };
        let visibility = shadow_visibility(
            shadow_ray,
            distance_to_light,
            &scene.objects,
            options.refractions,
        );
        if visibility == 0.0 {
            continue;
        }
        let reflect_dir = (-light_dir).reflect(normal).normalize();
        let specular = (-ray.direction).dot(reflect_dir).max(0.0).powf(48.0) * 0.35;
        let attenuation = 1.0 / (1.0 + 0.025 * distance_to_light * distance_to_light);
        let strength = light.brightness * attenuation * visibility;
        color += surface_color * light.color * (diffuse * strength);
        color += light.color * (specular * strength);
    }

    if depth >= options.max_depth {
        return color;
    }
    let reflective = options.reflections && hit.material.reflectivity > 0.0;
    let transmissive = options.refractions && hit.material.transmission > 0.0;
    if !reflective && !transmissive {
        return color;
    }
    let reflection_ray = Ray {
        origin: hit.point + normal * EPSILON,
        direction: ray.direction.reflect(normal).normalize(),
    };
    let reflected = trace_ray(reflection_ray, scene, options, depth + 1);
    if transmissive {
        let eta = if entering {
            1.0 / hit.material.ior
        } else {
            hit.material.ior
        };
        let transmission = hit.material.transmission;
        if let Some(direction) = ray.direction.refract(normal, eta) {
            let cosine = (-ray.direction).dot(normal).clamp(0.0, 1.0);
            let r0 = ((1.0 - hit.material.ior) / (1.0 + hit.material.ior)).powi(2);
            let fresnel = r0 + (1.0 - r0) * (1.0 - cosine).powi(5);
            let refracted = trace_ray(
                Ray {
                    origin: hit.point - normal * EPSILON,
                    direction,
                },
                scene,
                options,
                depth + 1,
            );
            color = color * (1.0 - transmission)
                + (reflected * fresnel + refracted * surface_color * (1.0 - fresnel))
                    * transmission;
        } else {
            color = color * (1.0 - transmission) + reflected * transmission;
        }
    } else {
        color = color * (1.0 - hit.material.reflectivity) + reflected * hit.material.reflectivity;
    }
    color
}

fn closest_hit(ray: Ray, objects: &[Object]) -> Option<Hit> {
    objects
        .iter()
        .filter_map(|object| object.intersect(ray))
        .min_by(|a, b| a.t.total_cmp(&b.t))
}

fn shadow_visibility(
    mut ray: Ray,
    mut distance: f64,
    objects: &[Object],
    refractions: bool,
) -> f64 {
    let mut visibility = 1.0;
    // Approximate straight-line transmission; this deliberately does not model caustics.
    for _ in 0..16 {
        let Some(hit) = closest_hit(ray, objects) else {
            return visibility;
        };
        if hit.t >= distance {
            return visibility;
        }
        if !refractions || hit.material.transmission == 0.0 {
            return 0.0;
        }
        visibility *= hit.material.transmission;
        distance -= hit.t + EPSILON;
        ray.origin = hit.point + ray.direction * EPSILON;
    }
    0.0
}

fn sky_color(direction: Vec3, base: Vec3) -> Vec3 {
    let t = 0.5 * (direction.y + 1.0);
    base * (1.0 - t) + Vec3::new(0.68, 0.78, 0.95) * t
}

pub(crate) fn render<W: Write>(
    scene: &Scene,
    options: &RenderOptions,
    mut writer: W,
) -> io::Result<()> {
    writeln!(writer, "P3\n{} {}\n255", options.width, options.height)?;
    for y in 0..options.height {
        for x in 0..options.width {
            let ray = scene.camera.ray_for_sample(
                x as f64 + 0.5,
                y as f64 + 0.5,
                options.width,
                options.height,
            );
            let color = trace_ray(ray, scene, options, 0);
            let [r, g, b] = [color.x, color.y, color.z]
                .map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8);
            writeln!(writer, "{r} {g} {b}")?;
        }
    }
    writer.flush()
}
