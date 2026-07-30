use super::Octonion;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OctonionF32 {
    pub r: f32,
    pub e1: f32,
    pub e2: f32,
    pub e3: f32,
    pub e4: f32,
    pub e5: f32,
    pub e6: f32,
    pub e7: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OctonionF64 {
    pub r: f64,
    pub e1: f64,
    pub e2: f64,
    pub e3: f64,
    pub e4: f64,
    pub e5: f64,
    pub e6: f64,
    pub e7: f64,
}

impl From<Octonion<f32>> for OctonionF32 {
    #[inline]
    fn from(o: Octonion<f32>) -> Self {
        Self { r: o.r, e1: o.e1, e2: o.e2, e3: o.e3, e4: o.e4, e5: o.e5, e6: o.e6, e7: o.e7 }
    }
}

impl From<OctonionF32> for Octonion<f32> {
    #[inline]
    fn from(o: OctonionF32) -> Self {
        Self { r: o.r, e1: o.e1, e2: o.e2, e3: o.e3, e4: o.e4, e5: o.e5, e6: o.e6, e7: o.e7 }
    }
}

impl From<Octonion<f64>> for OctonionF64 {
    #[inline]
    fn from(o: Octonion<f64>) -> Self {
        Self { r: o.r, e1: o.e1, e2: o.e2, e3: o.e3, e4: o.e4, e5: o.e5, e6: o.e6, e7: o.e7 }
    }
}

impl From<OctonionF64> for Octonion<f64> {
    #[inline]
    fn from(o: OctonionF64) -> Self {
        Self { r: o.r, e1: o.e1, e2: o.e2, e3: o.e3, e4: o.e4, e5: o.e5, e6: o.e6, e7: o.e7 }
    }
}

#[no_mangle]
pub extern "C" fn octonion_mul_f32(a: OctonionF32, b: OctonionF32) -> OctonionF32 {
    let oa: Octonion<f32> = a.into();
    let ob: Octonion<f32> = b.into();
    (oa * ob).into()
}

#[no_mangle]
pub extern "C" fn octonion_add_f32(a: OctonionF32, b: OctonionF32) -> OctonionF32 {
    let oa: Octonion<f32> = a.into();
    let ob: Octonion<f32> = b.into();
    (oa + ob).into()
}

#[no_mangle]
pub extern "C" fn octonion_sub_f32(a: OctonionF32, b: OctonionF32) -> OctonionF32 {
    let oa: Octonion<f32> = a.into();
    let ob: Octonion<f32> = b.into();
    (oa - ob).into()
}

#[no_mangle]
pub extern "C" fn octonion_conj_f32(a: OctonionF32) -> OctonionF32 {
    let oa: Octonion<f32> = a.into();
    oa.conj().into()
}

#[no_mangle]
pub extern "C" fn octonion_norm_f32(a: OctonionF32) -> f32 {
    let oa: Octonion<f32> = a.into();
    oa.norm()
}

#[no_mangle]
pub extern "C" fn octonion_norm_sq_f32(a: OctonionF32) -> f32 {
    let oa: Octonion<f32> = a.into();
    oa.norm_sq()
}

#[no_mangle]
pub extern "C" fn octonion_inv_f32(a: OctonionF32) -> OctonionF32 {
    let oa: Octonion<f32> = a.into();
    match oa.inv() {
        Some(inv) => inv.into(),
        None => {
            OctonionF32 { r: 0.0, e1: 0.0, e2: 0.0, e3: 0.0, e4: 0.0, e5: 0.0, e6: 0.0, e7: 0.0 }
        }
    }
}

#[no_mangle]
pub extern "C" fn octonion_mul_f64(a: OctonionF64, b: OctonionF64) -> OctonionF64 {
    let oa: Octonion<f64> = a.into();
    let ob: Octonion<f64> = b.into();
    (oa * ob).into()
}

#[no_mangle]
pub extern "C" fn octonion_add_f64(a: OctonionF64, b: OctonionF64) -> OctonionF64 {
    let oa: Octonion<f64> = a.into();
    let ob: Octonion<f64> = b.into();
    (oa + ob).into()
}

#[no_mangle]
pub extern "C" fn octonion_conj_f64(a: OctonionF64) -> OctonionF64 {
    let oa: Octonion<f64> = a.into();
    oa.conj().into()
}

#[no_mangle]
pub extern "C" fn octonion_norm_f64(a: OctonionF64) -> f64 {
    let oa: Octonion<f64> = a.into();
    oa.norm()
}

macro_rules! check_batch_count {
    ($count:expr, $op_name:expr) => {
        if $count == 0 {
            return;
        }
        if $count > usize::MAX / 8 {
            return;
        }
    };
}

#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn octonion_mul_batch_f32(
    a: *const OctonionF32,
    b: *const OctonionF32,
    out: *mut OctonionF32,
    count: usize,
) {
    if a.is_null() || b.is_null() || out.is_null() {
        return;
    }
    check_batch_count!(count, "mul_batch");
    for i in 0..count {
        let oa: Octonion<f32> = (*a.add(i)).into();
        let ob: Octonion<f32> = (*b.add(i)).into();
        *out.add(i) = (oa * ob).into();
    }
}

#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn octonion_norm_batch_f32(
    a: *const OctonionF32,
    out: *mut f32,
    count: usize,
) {
    if a.is_null() || out.is_null() {
        return;
    }
    check_batch_count!(count, "norm_batch");
    for i in 0..count {
        let oa: Octonion<f32> = (*a.add(i)).into();
        *out.add(i) = oa.norm();
    }
}

#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn octonion_to_flat_f32(
    octs: *const OctonionF32,
    out: *mut f32,
    count: usize,
) {
    if octs.is_null() || out.is_null() {
        return;
    }
    check_batch_count!(count, "to_flat");
    for i in 0..count {
        let o = &*octs.add(i);
        let base = out.add(i * 8);
        *base = o.r;
        *base.add(1) = o.e1;
        *base.add(2) = o.e2;
        *base.add(3) = o.e3;
        *base.add(4) = o.e4;
        *base.add(5) = o.e5;
        *base.add(6) = o.e6;
        *base.add(7) = o.e7;
    }
}

#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn octonion_from_flat_f32(
    flat: *const f32,
    out: *mut OctonionF32,
    count: usize,
) {
    if flat.is_null() || out.is_null() {
        return;
    }
    check_batch_count!(count, "from_flat");
    for i in 0..count {
        let base = flat.add(i * 8);
        *out.add(i) = OctonionF32 {
            r: *base,
            e1: *base.add(1),
            e2: *base.add(2),
            e3: *base.add(3),
            e4: *base.add(4),
            e5: *base.add(5),
            e6: *base.add(6),
            e7: *base.add(7),
        };
    }
}
