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
    pub(crate) samples: usize,
    pub(crate) threads: usize,
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

fn pixel(scene: &Scene, options: &RenderOptions, x: usize, y: usize) -> [u8; 3] {
    let mut color = Vec3::ZERO;
    for sy in 0..options.samples {
        for sx in 0..options.samples {
            let x = x as f64 + (sx as f64 + 0.5) / options.samples as f64;
            let y = y as f64 + (sy as f64 + 0.5) / options.samples as f64;
            color += trace_ray(
                scene
                    .camera
                    .ray_for_sample(x, y, options.width, options.height),
                scene,
                options,
                0,
            );
        }
    }
    let color = color / (options.samples * options.samples) as f64;
    // Average in linear space, then tone-map highlights and encode for display.
    [color.x, color.y, color.z].map(|channel| {
        let linear = channel.max(0.0);
        let mapped = linear / (1.0 + linear);
        (mapped.powf(1.0 / 2.2) * 255.0).round() as u8
    })
}

pub(crate) fn render<W: Write>(
    scene: &Scene,
    options: &RenderOptions,
    mut writer: W,
) -> io::Result<()> {
    let count = options.width * options.height;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(count)
        .map_err(|e| io::Error::other(format!("cannot allocate image: {e}")))?;
    pixels.resize(count, [0u8; 3]);
    let rows_per_worker = options.height.div_ceil(options.threads);
    let chunk_size = rows_per_worker * options.width;
    std::thread::scope(|scope| -> io::Result<()> {
        let mut workers = Vec::new();
        for (index, chunk) in pixels.chunks_mut(chunk_size).enumerate() {
            let first_y = index * rows_per_worker;
            let worker = std::thread::Builder::new().spawn_scoped(scope, move || {
                for (offset, output) in chunk.iter_mut().enumerate() {
                    *output = pixel(
                        scene,
                        options,
                        offset % options.width,
                        first_y + offset / options.width,
                    );
                }
            })?;
            workers.push(worker);
        }
        for worker in workers {
            worker
                .join()
                .map_err(|_| io::Error::other("render worker panicked"))?;
        }
        Ok(())
    })?;
    writeln!(writer, "P3\n{} {}\n255", options.width, options.height)?;
    for [r, g, b] in pixels {
        writeln!(writer, "{r} {g} {b}")?;
    }
    writer.flush()
}
