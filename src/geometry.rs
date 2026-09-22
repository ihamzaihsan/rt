use crate::math::{Ray, Vec3};

const EPSILON: f64 = 1.0e-4;

#[derive(Clone, Copy)]
pub(crate) struct Material {
    pub(crate) color: Vec3,
    pub(crate) reflectivity: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct Hit {
    pub(crate) t: f64,
    pub(crate) point: Vec3,
    pub(crate) normal: Vec3,
    pub(crate) material: Material,
}

pub(crate) enum Object {
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
    pub(crate) fn intersect(&self, ray: Ray) -> Option<Hit> {
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
    if denom.abs() < 1.0e-12 {
        return None;
    }

    let t = (point - ray.origin).dot(normal) / denom;
    if t <= EPSILON {
        return None;
    }

    Some(Hit {
        t,
        point: ray.at(t),
        normal,
        material,
    })
}

fn intersect_cube(ray: Ray, min: Vec3, max: Vec3, material: Material) -> Option<Hit> {
    let mut t_min = -f64::INFINITY;
    let mut t_max = f64::INFINITY;
    let mut hit_normal = Vec3::ZERO;
    let mut exit_normal = Vec3::ZERO;

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
        if direction.abs() < 1.0e-12 {
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
        if t1 < t_max {
            t_max = t1;
            exit_normal = far_normal;
        }
        if t_min > t_max {
            return None;
        }
    }

    let (t, hit_normal) = if t_min > EPSILON {
        (t_min, hit_normal)
    } else {
        (t_max, exit_normal)
    };
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

    if a.abs() > 1.0e-12 {
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
        if ray.direction.y.abs() < 1.0e-12 {
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
                    normal,
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
