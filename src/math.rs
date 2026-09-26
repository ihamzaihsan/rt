#[derive(Clone, Copy, Debug)]
pub(crate) struct Vec3 {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

impl Vec3 {
    pub(crate) const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub(crate) const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub(crate) fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub(crate) fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub(crate) fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub(crate) fn normalize(self) -> Self {
        let length = self.length();
        if length == 0.0 { self } else { self / length }
    }

    pub(crate) fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }

    /// Snell's law; `normal` faces the incoming ray and `eta` is incident / transmitted IOR.
    pub(crate) fn refract(self, normal: Self, eta: f64) -> Option<Self> {
        let cosine = (-self).dot(normal).clamp(0.0, 1.0);
        let perpendicular = (self + normal * cosine) * eta;
        let parallel_squared = 1.0 - perpendicular.dot(perpendicular);
        if parallel_squared < 0.0 {
            None // Total internal reflection.
        } else {
            Some((perpendicular - normal * parallel_squared.sqrt()).normalize())
        }
    }

    pub(crate) fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
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
pub(crate) struct Ray {
    pub(crate) origin: Vec3,
    pub(crate) direction: Vec3,
}

impl Ray {
    pub(crate) fn at(self, t: f64) -> Vec3 {
        self.origin + self.direction * t
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
