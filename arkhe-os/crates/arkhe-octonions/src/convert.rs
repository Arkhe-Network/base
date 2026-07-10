use super::Octonion;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Quaternion<T> {
    pub w: T,
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Copy> From<Octonion<T>> for Quaternion<T> {
    fn from(o: Octonion<T>) -> Self {
        Self { w: o.r, x: o.e1, y: o.e2, z: o.e3 }
    }
}

impl<T: Copy + Default> From<Quaternion<T>> for Octonion<T> {
    fn from(q: Quaternion<T>) -> Self {
        Self {
            r: q.w,
            e1: q.x,
            e2: q.y,
            e3: q.z,
            e4: T::default(),
            e5: T::default(),
            e6: T::default(),
            e7: T::default(),
        }
    }
}

pub fn to_rotation_matrix_f(o: &Octonion<f32>) -> [f32; 9] {
    let (w, x, y, z) = (o.r, o.e1, o.e2, o.e3);
    let t = 2.0;
    [
        1.0 - t * (y * y + z * z),
        t * (x * y - w * z),
        t * (x * z + w * y),
        t * (x * y + w * z),
        1.0 - t * (x * x + z * z),
        t * (y * z - w * x),
        t * (x * z - w * y),
        t * (y * z + w * x),
        1.0 - t * (x * x + y * y),
    ]
}

pub fn to_rotation_matrix_d(o: &Octonion<f64>) -> [f64; 9] {
    let (w, x, y, z) = (o.r, o.e1, o.e2, o.e3);
    let t = 2.0;
    [
        1.0 - t * (y * y + z * z),
        t * (x * y - w * z),
        t * (x * z + w * y),
        t * (x * y + w * z),
        1.0 - t * (x * x + z * z),
        t * (y * z - w * x),
        t * (x * z - w * y),
        t * (y * z + w * x),
        1.0 - t * (x * x + y * y),
    ]
}

pub fn to_euler_f(o: &Octonion<f32>) -> (f32, f32, f32) {
    let (w, x, y, z) = (o.r, o.e1, o.e2, o.e3);
    let sinr = 2.0 * (w * x + y * z);
    let cosr = 1.0 - 2.0 * (x * x + y * y);
    let roll = libm::atan2f(sinr, cosr);

    let sinp = 2.0 * (w * y - z * x);
    let pitch = if sinp.abs() >= 1.0 {
        if sinp > 0.0 { core::f32::consts::FRAC_PI_2 } else { -core::f32::consts::FRAC_PI_2 }
    } else {
        libm::asinf(sinp)
    };

    let siny = 2.0 * (w * z + x * y);
    let cosy = 1.0 - 2.0 * (y * y + z * z);
    let yaw = libm::atan2f(siny, cosy);

    (roll, pitch, yaw)
}

pub fn to_euler_d(o: &Octonion<f64>) -> (f64, f64, f64) {
    let (w, x, y, z) = (o.r, o.e1, o.e2, o.e3);
    let sinr = 2.0 * (w * x + y * z);
    let cosr = 1.0 - 2.0 * (x * x + y * y);
    let roll = libm::atan2(sinr, cosr);

    let sinp = 2.0 * (w * y - z * x);
    let pitch = if sinp.abs() >= 1.0 {
        if sinp > 0.0 { core::f64::consts::FRAC_PI_2 } else { -core::f64::consts::FRAC_PI_2 }
    } else {
        libm::asin(sinp)
    };

    let siny = 2.0 * (w * z + x * y);
    let cosy = 1.0 - 2.0 * (y * y + z * z);
    let yaw = libm::atan2(siny, cosy);

    (roll, pitch, yaw)
}
