use std::env;
use std::f64::INFINITY;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::process;

const EPSILON: f64 = 1.0e-4;

#[derive(Clone, Copy, Debug)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    fn normalize(self) -> Self {
        let length = self.length();
        if length == 0.0 { self } else { self / length }
    }

    fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }

    fn clamp01(self) -> Self {
        Self::new(
            self.x.clamp(0.0, 1.0),
            self.y.clamp(0.0, 1.0),
            self.z.clamp(0.0, 1.0),
        )
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl std::ops::Mul for Vec3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl std::ops::Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

#[derive(Clone, Copy)]
struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn at(self, t: f64) -> Vec3 {
        self.origin + self.direction * t
    }
}

#[derive(Clone, Copy)]
struct Material {
    color: Vec3,
    reflectivity: f64,
}

#[derive(Clone, Copy)]
struct Hit {
    t: f64,
    point: Vec3,
    normal: Vec3,
    material: Material,
}

enum Object {
    Sphere {
        center: Vec3,
        radius: f64,
        material: Material,
    },
    Plane {
        point: Vec3,
        normal: Vec3,
        material: Material,
    },
    Cube {
        min: Vec3,
        max: Vec3,
        material: Material,
    },
    Cylinder {
        center: Vec3,
        radius: f64,
        height: f64,
        material: Material,
    },
}

impl Object {
    fn intersect(&self, ray: Ray) -> Option<Hit> {
        match *self {
            Object::Sphere {
                center,
                radius,
                material,
            } => intersect_sphere(ray, center, radius, material),
            Object::Plane {
                point,
                normal,
                material,
            } => intersect_plane(ray, point, normal, material),
            Object::Cube { min, max, material } => intersect_cube(ray, min, max, material),
            Object::Cylinder {
                center,
                radius,
                height,
                material,
            } => intersect_cylinder(ray, center, radius, height, material),
        }
    }
}

fn intersect_sphere(ray: Ray, center: Vec3, radius: f64, material: Material) -> Option<Hit> {
    let oc = ray.origin - center;
    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let root = discriminant.sqrt();
    let t1 = (-b - root) / (2.0 * a);
    let t2 = (-b + root) / (2.0 * a);
    let t = if t1 > EPSILON { t1 } else { t2 };
    if t <= EPSILON {
        return None;
    }

    let point = ray.at(t);
    Some(Hit {
        t,
        point,
        normal: (point - center).normalize(),
        material,
    })
}

fn intersect_plane(ray: Ray, point: Vec3, normal: Vec3, material: Material) -> Option<Hit> {
    let normal = normal.normalize();
    let denom = normal.dot(ray.direction);
    if denom.abs() < EPSILON {
        return None;
    }

    let t = (point - ray.origin).dot(normal) / denom;
    if t <= EPSILON {
        return None;
    }

    Some(Hit {
        t,
        point: ray.at(t),
        normal: if denom < 0.0 { normal } else { -normal },
        material,
    })
}

fn intersect_cube(ray: Ray, min: Vec3, max: Vec3, material: Material) -> Option<Hit> {
    let mut t_min = -INFINITY;
    let mut t_max = INFINITY;
    let mut hit_normal = Vec3::ZERO;

    let axes = [
        (
            ray.origin.x,
            ray.direction.x,
            min.x,
            max.x,
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        ),
        (
            ray.origin.y,
            ray.direction.y,
            min.y,
            max.y,
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ),
        (
            ray.origin.z,
            ray.direction.z,
            min.z,
            max.z,
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
        ),
    ];

    for (origin, direction, axis_min, axis_max, min_normal, max_normal) in axes {
        if direction.abs() < EPSILON {
            if origin < axis_min || origin > axis_max {
                return None;
            }
            continue;
        }

        let inv = 1.0 / direction;
        let mut t0 = (axis_min - origin) * inv;
        let mut t1 = (axis_max - origin) * inv;
        let mut near_normal = min_normal;
        let mut far_normal = max_normal;

        if inv < 0.0 {
            std::mem::swap(&mut t0, &mut t1);
            std::mem::swap(&mut near_normal, &mut far_normal);
        }

        if t0 > t_min {
            t_min = t0;
            hit_normal = near_normal;
        }
        t_max = t_max.min(t1);
        if t_min > t_max {
            return None;
        }
    }

    let t = if t_min > EPSILON { t_min } else { t_max };
    if t <= EPSILON {
        return None;
    }

    Some(Hit {
        t,
        point: ray.at(t),
        normal: hit_normal,
        material,
    })
}

fn intersect_cylinder(
    ray: Ray,
    center: Vec3,
    radius: f64,
    height: f64,
    material: Material,
) -> Option<Hit> {
    let half_height = height / 2.0;
    let oc = ray.origin - center;
    let a = ray.direction.x * ray.direction.x + ray.direction.z * ray.direction.z;
    let b = 2.0 * (oc.x * ray.direction.x + oc.z * ray.direction.z);
    let c = oc.x * oc.x + oc.z * oc.z - radius * radius;
    let mut best: Option<Hit> = None;

    if a.abs() > EPSILON {
        let discriminant = b * b - 4.0 * a * c;
        if discriminant >= 0.0 {
            let root = discriminant.sqrt();
            for t in [(-b - root) / (2.0 * a), (-b + root) / (2.0 * a)] {
                let y = oc.y + t * ray.direction.y;
                if t > EPSILON && y >= -half_height && y <= half_height {
                    let point = ray.at(t);
                    let normal = Vec3::new(point.x - center.x, 0.0, point.z - center.z).normalize();
                    best = nearest_hit(
                        best,
                        Hit {
                            t,
                            point,
                            normal,
                            material,
                        },
                    );
                }
            }
        }
    }

    for (cap_y, normal) in [
        (center.y - half_height, Vec3::new(0.0, -1.0, 0.0)),
        (center.y + half_height, Vec3::new(0.0, 1.0, 0.0)),
    ] {
        if ray.direction.y.abs() < EPSILON {
            continue;
        }
        let t = (cap_y - ray.origin.y) / ray.direction.y;
        let point = ray.at(t);
        let dx = point.x - center.x;
        let dz = point.z - center.z;
        if t > EPSILON && dx * dx + dz * dz <= radius * radius {
            best = nearest_hit(
                best,
                Hit {
                    t,
                    point,
                    normal: if ray.direction.dot(normal) < 0.0 {
                        normal
                    } else {
                        -normal
                    },
                    material,
                },
            );
        }
    }

    best
}

fn nearest_hit(current: Option<Hit>, candidate: Hit) -> Option<Hit> {
    match current {
        Some(hit) if hit.t <= candidate.t => Some(hit),
        _ => Some(candidate),
    }
}

#[derive(Clone, Copy)]
struct Light {
    position: Vec3,
    color: Vec3,
    brightness: f64,
}

struct Scene {
    objects: Vec<Object>,
    lights: Vec<Light>,
    camera: Camera,
    background: Vec3,
    ambient: f64,
}

struct Camera {
    origin: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    fov_radians: f64,
}

impl Camera {
    fn look_at(origin: Vec3, target: Vec3, up_hint: Vec3, fov_degrees: f64) -> Self {
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

    fn ray_for_pixel(&self, x: usize, y: usize, width: usize, height: usize) -> Ray {
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

struct RenderOptions {
    width: usize,
    height: usize,
    reflections: bool,
    max_depth: usize,
}

fn trace_ray(ray: Ray, scene: &Scene, options: &RenderOptions, depth: usize) -> Vec3 {
    let Some(hit) = closest_hit(ray, &scene.objects) else {
        return sky_color(ray.direction, scene.background);
    };

    let mut color = hit.material.color * scene.ambient;
    for light in &scene.lights {
        let to_light = light.position - hit.point;
        let distance_to_light = to_light.length();
        let light_dir = to_light / distance_to_light;
        let shadow_ray = Ray {
            origin: hit.point + hit.normal * EPSILON,
            direction: light_dir,
        };

        let in_shadow = closest_hit(shadow_ray, &scene.objects)
            .is_some_and(|shadow_hit| shadow_hit.t < distance_to_light);
        if in_shadow {
            continue;
        }

        let diffuse = hit.normal.dot(light_dir).max(0.0);
        let view_dir = -ray.direction;
        let reflect_dir = (-light_dir).reflect(hit.normal).normalize();
        let specular = view_dir.dot(reflect_dir).max(0.0).powf(48.0) * 0.35;
        let attenuation = 1.0 / (1.0 + 0.025 * distance_to_light * distance_to_light);
        let light_strength = light.brightness * attenuation;

        color += hit.material.color * light.color * (diffuse * light_strength);
        color += light.color * (specular * light_strength);
    }

    if options.reflections && depth < options.max_depth && hit.material.reflectivity > 0.0 {
        let reflection_ray = Ray {
            origin: hit.point + hit.normal * EPSILON,
            direction: ray.direction.reflect(hit.normal).normalize(),
        };
        let reflected = trace_ray(reflection_ray, scene, options, depth + 1);
        color = color * (1.0 - hit.material.reflectivity) + reflected * hit.material.reflectivity;
    }

    color.clamp01()
}

fn closest_hit(ray: Ray, objects: &[Object]) -> Option<Hit> {
    objects
        .iter()
        .filter_map(|object| object.intersect(ray))
        .min_by(|a, b| a.t.total_cmp(&b.t))
}

fn sky_color(direction: Vec3, base: Vec3) -> Vec3 {
    let t = 0.5 * (direction.y + 1.0);
    base * (1.0 - t) + Vec3::new(0.68, 0.78, 0.95) * t
}

fn render<W: Write>(scene: &Scene, options: &RenderOptions, mut writer: W) -> io::Result<()> {
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

    Ok(())
}

fn material(color: Vec3, reflectivity: f64) -> Material {
    Material {
        color,
        reflectivity,
    }
}

fn scene_by_name(
    name: &str,
    brightness: Option<f64>,
    camera_override: Option<Vec3>,
) -> Option<Scene> {
    match name {
        "sphere" => {
            let camera_pos = camera_override.unwrap_or(Vec3::new(0.0, 1.0, 4.5));
            Some(Scene {
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
                    Vec3::new(0.0, 0.8, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                    50.0,
                ),
                background: Vec3::new(0.08, 0.10, 0.14),
                ambient: 0.12,
            })
        }
        "cube-plane" => {
            let camera_pos = camera_override.unwrap_or(Vec3::new(2.4, 1.8, 5.2));
            Some(Scene {
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
                    Vec3::new(0.0, 0.25, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                    48.0,
                ),
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
            Some(Scene {
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
                    Vec3::new(0.0, 0.0, -0.35),
                    Vec3::new(0.0, 1.0, 0.0),
                    50.0,
                ),
                background: Vec3::new(0.08, 0.09, 0.12),
                ambient: 0.11,
            })
        }
        _ => None,
    }
}

#[derive(Default)]
struct Args {
    scene: String,
    width: usize,
    height: usize,
    output: Option<String>,
    brightness: Option<f64>,
    camera: Option<Vec3>,
    reflections: bool,
    help: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        scene: "sphere".to_string(),
        width: 800,
        height: 600,
        reflections: false,
        ..Args::default()
    };
    let mut iter = env::args().skip(1);

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--scene" => args.scene = next_value(&mut iter, "--scene")?,
            "--width" => args.width = parse_next(&mut iter, "--width")?,
            "--height" => args.height = parse_next(&mut iter, "--height")?,
            "--output" | "-o" => args.output = Some(next_value(&mut iter, "--output")?),
            "--brightness" => args.brightness = Some(parse_next(&mut iter, "--brightness")?),
            "--camera" => args.camera = Some(parse_vec3(&next_value(&mut iter, "--camera")?)?),
            "--reflections" | "-r" => args.reflections = true,
            "--help" | "-h" => args.help = true,
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
    }

    if args.width == 0 || args.height == 0 {
        return Err("width and height must be greater than 0".to_string());
    }

    Ok(args)
}

fn next_value(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    iter.next()
        .ok_or_else(|| format!("missing value after {flag}"))
}

fn parse_next<T: std::str::FromStr>(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<T, String> {
    next_value(iter, flag)?
        .parse()
        .map_err(|_| format!("invalid value for {flag}"))
}

fn parse_vec3(value: &str) -> Result<Vec3, String> {
    let parts: Vec<_> = value.split(',').collect();
    if parts.len() != 3 {
        return Err("camera must use x,y,z format, for example --camera 3,2,5".to_string());
    }
    let x = parts[0].parse().map_err(|_| "invalid camera x value")?;
    let y = parts[1].parse().map_err(|_| "invalid camera y value")?;
    let z = parts[2].parse().map_err(|_| "invalid camera z value")?;
    Ok(Vec3::new(x, y, z))
}

fn print_help() {
    eprintln!(
        "Usage: cargo run --release -- [options]\n\n\
         Options:\n\
           --scene <name>          sphere | cube-plane | all | all-alt (default: sphere)\n\
           --width <pixels>        Output width (default: 800)\n\
           --height <pixels>       Output height (default: 600)\n\
           --output, -o <file>     Write PPM to a file instead of stdout\n\
           --brightness <number>   Override light brightness for the selected scene\n\
           --camera <x,y,z>        Override camera position while keeping scene target\n\
           --reflections, -r       Enable reflective materials\n\
           --help, -h              Show this help\n\n\
         Example: cargo run --release -- --scene all --reflections > output.ppm"
    );
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    if args.help {
        print_help();
        return Ok(());
    }

    let scene = scene_by_name(&args.scene, args.brightness, args.camera).ok_or_else(|| {
        format!(
            "unknown scene '{}'. Use --help for valid scenes.",
            args.scene
        )
    })?;
    let options = RenderOptions {
        width: args.width,
        height: args.height,
        reflections: args.reflections,
        max_depth: 3,
    };

    match args.output {
        Some(path) => {
            let file =
                File::create(&path).map_err(|err| format!("failed to create {path}: {err}"))?;
            render(&scene, &options, BufWriter::new(file))
                .map_err(|err| format!("failed to render {path}: {err}"))?;
        }
        None => {
            let stdout = io::stdout();
            render(&scene, &options, BufWriter::new(stdout.lock()))
                .map_err(|err| format!("failed to write ppm: {err}"))?;
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("rt: {err}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::Vec3;

    fn assert_vec3_close(actual: Vec3, expected: Vec3) {
        let delta = actual - expected;
        assert!(
            delta.length() < 1.0e-10,
            "expected ({}, {}, {}), got ({}, {}, {})",
            expected.x,
            expected.y,
            expected.z,
            actual.x,
            actual.y,
            actual.z
        );
    }

    #[test]
    fn vec3_arithmetic_and_scaling_work() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, -1.0, 0.5);

        assert_vec3_close(a + b, Vec3::new(5.0, 1.0, 3.5));
        assert_vec3_close(a - b, Vec3::new(-3.0, 3.0, 2.5));
        assert_vec3_close(a * 2.0, Vec3::new(2.0, 4.0, 6.0));
        assert_vec3_close(a / 2.0, Vec3::new(0.5, 1.0, 1.5));
    }

    #[test]
    fn vec3_dot_cross_and_normalize_work() {
        let x_axis = Vec3::new(1.0, 0.0, 0.0);
        let y_axis = Vec3::new(0.0, 1.0, 0.0);
        let diagonal = Vec3::new(3.0, 4.0, 0.0);

        assert_eq!(x_axis.dot(y_axis), 0.0);
        assert_vec3_close(x_axis.cross(y_axis), Vec3::new(0.0, 0.0, 1.0));
        assert!((diagonal.length() - 5.0).abs() < 1.0e-10);
        assert_vec3_close(diagonal.normalize(), Vec3::new(0.6, 0.8, 0.0));
    }
}
