use crate::gdid::{Gdid, Namespace};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GdidFFIResult(i32);

impl GdidFFIResult {
    pub const OK: Self = Self(0);
    pub const NULL_POINTER: Self = Self(-1);
    pub const INVALID_NAMESPACE: Self = Self(-2);
    pub const SERIALIZATION: Self = Self(-3);
    pub const BUFFER_SIZE: Self = Self(-4);
    pub const INVALID_PUBKEY: Self = Self(-5);
    pub const EXPIRED: Self = Self(-6);
    pub const SIGNATURE_INVALID: Self = Self(-7);

    #[inline]
    pub fn code(self) -> i32 {
        self.0
    }

    #[inline]
    pub fn is_ok(self) -> bool {
        self.0 == 0
    }
}

fn parse_namespace(ns: u8) -> Option<Namespace> {
    match ns {
        0x00 => Some(Namespace::ArkheGlobal),
        0x01 => Some(Namespace::HubbleNetwork),
        0x02 => Some(Namespace::Oem),
        _ => None,
    }
}

#[no_mangle]
pub extern "C" fn gdid_derive(
    fingerprint: *const u8,
    namespace: u8,
    nonce: u32,
    out_gdid: *mut u8,
) -> GdidFFIResult {
    if fingerprint.is_null() || out_gdid.is_null() {
        return GdidFFIResult::NULL_POINTER;
    }

    let fp_array: [u8; 32] = unsafe {
        let s = core::slice::from_raw_parts(fingerprint, 32);
        match <[u8; 32]>::try_from(s) {
            Ok(a) => a,
            Err(_) => return GdidFFIResult::BUFFER_SIZE,
        }
    };

    let ns = match parse_namespace(namespace) {
        Some(n) => n,
        None => return GdidFFIResult::INVALID_NAMESPACE,
    };

    let gdid = Gdid::from_fingerprint(&fp_array, ns, nonce);

    unsafe {
        core::ptr::copy_nonoverlapping(gdid.as_bytes().as_ptr(), out_gdid, 32);
    }

    GdidFFIResult::OK
}

#[no_mangle]
pub extern "C" fn gdid_decompose(
    gdid: *const u8,
    out_version: *mut u8,
    out_namespace: *mut u8,
    out_nonce: *mut u8,
    out_hw_hash: *mut u8,
) -> GdidFFIResult {
    if gdid.is_null()
        || out_version.is_null()
        || out_namespace.is_null()
        || out_nonce.is_null()
        || out_hw_hash.is_null()
    {
        return GdidFFIResult::NULL_POINTER;
    }

    unsafe {
        let bytes = core::slice::from_raw_parts(gdid, 32);
        core::ptr::write(out_version, bytes[0]);
        core::ptr::write(out_namespace, bytes[1]);
        core::ptr::copy_nonoverlapping(bytes.as_ptr().add(22), out_nonce, 4);
        core::ptr::copy_nonoverlapping(bytes.as_ptr().add(2), out_hw_hash, 20);
    }

    GdidFFIResult::OK
}

#[no_mangle]
pub extern "C" fn gdid_equal(a: *const u8, b: *const u8) -> u8 {
    if a.is_null() || b.is_null() {
        return 0;
    }

    unsafe {
        let sa = core::slice::from_raw_parts(a, 32);
        let sb = core::slice::from_raw_parts(b, 32);
        bool::from(subtle::ConstantTimeEq::ct_eq(sa, sb)) as u8
    }
}

#[no_mangle]
pub extern "C" fn gdid_to_base58(gdid: *const u8, out_buf: *mut u8, buf_len: usize) -> i32 {
    if gdid.is_null() || out_buf.is_null() {
        return GdidFFIResult::NULL_POINTER.code();
    }
    if buf_len < 80 {
        return GdidFFIResult::BUFFER_SIZE.code();
    }

    let arr: [u8; 32] = unsafe {
        let s = core::slice::from_raw_parts(gdid, 32);
        match <[u8; 32]>::try_from(s) {
            Ok(a) => a,
            Err(_) => return GdidFFIResult::BUFFER_SIZE.code(),
        }
    };

    let gdid_obj = Gdid::from_raw(&arr);
    let s = gdid_obj.to_base58();

    unsafe {
        core::ptr::copy_nonoverlapping(s.as_ptr(), out_buf, s.len());
        *out_buf.add(s.len()) = 0;
    }

    s.len() as i32
}

#[no_mangle]
pub extern "C" fn gdid_to_hex(gdid: *const u8, out_buf: *mut u8, buf_len: usize) -> i32 {
    if gdid.is_null() || out_buf.is_null() {
        return GdidFFIResult::NULL_POINTER.code();
    }
    if buf_len < 65 {
        return GdidFFIResult::BUFFER_SIZE.code();
    }

    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    let bytes = unsafe { core::slice::from_raw_parts(gdid, 32) };

    unsafe {
        for (i, &byte) in bytes.iter().enumerate() {
            let hi = (byte >> 4) as usize;
            let lo = (byte & 0x0f) as usize;
            *out_buf.add(i * 2) = HEX_CHARS[hi];
            *out_buf.add(i * 2 + 1) = HEX_CHARS[lo];
        }
        *out_buf.add(64) = 0;
    }

    64
}

#[no_mangle]
pub extern "C" fn gdid_wipe(gdid: *mut u8) {
    if gdid.is_null() {
        return;
    }
    unsafe {
        core::ptr::write_bytes(gdid, 0, 32);
    }
}
