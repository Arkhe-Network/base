#![allow(unused_imports)]

use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[cfg(feature = "wasm")]
use js_sys::Float32Array;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use super::Octonion;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct OctonionF32Wasm(Octonion<f32>);

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl OctonionF32Wasm {
    #[wasm_bindgen(constructor)]
    pub fn new(r: f32, e1: f32, e2: f32, e3: f32, e4: f32, e5: f32, e6: f32, e7: f32) -> Self {
        Self(Octonion::new(r, e1, e2, e3, e4, e5, e6, e7))
    }

    #[wasm_bindgen]
    pub fn zero() -> Self {
        Self(Octonion::default())
    }

    #[wasm_bindgen]
    pub fn real(r: f32) -> Self {
        Self(Octonion::real(r))
    }

    #[wasm_bindgen]
    pub fn mul(&self, other: &OctonionF32Wasm) -> Self {
        Self(self.0 * other.0)
    }

    #[wasm_bindgen]
    pub fn add(&self, other: &OctonionF32Wasm) -> Self {
        Self(self.0 + other.0)
    }

    #[wasm_bindgen]
    pub fn sub(&self, other: &OctonionF32Wasm) -> Self {
        Self(self.0 - other.0)
    }

    #[wasm_bindgen]
    pub fn conj(&self) -> Self {
        Self(self.0.conj())
    }

    #[wasm_bindgen]
    pub fn norm(&self) -> f32 {
        self.0.norm()
    }

    #[wasm_bindgen]
    pub fn norm_sq(&self) -> f32 {
        self.0.norm_sq()
    }

    #[wasm_bindgen(getter)]
    pub fn r(&self) -> f32 {
        self.0.r
    }

    #[wasm_bindgen]
    pub fn components(&self) -> Float32Array {
        let arr = self.0.to_array();
        let view = Float32Array::view(&arr);
        let result = Float32Array::new_with_length(8);
        result.set(&view, 0);
        result
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = "octonionMulBatch")]
pub fn octonion_mul_batch(a: &Float32Array, b: &Float32Array) -> Float32Array {
    let a_slice = a.to_vec();
    let b_slice = b.to_vec();
    let n = a_slice.len() / 8;
    let out_len = n * 8;

    let mut out = Vec::with_capacity(out_len);
    for i in 0..n {
        let base = i * 8;
        let oa = Octonion::from_array([
            a_slice[base],
            a_slice[base + 1],
            a_slice[base + 2],
            a_slice[base + 3],
            a_slice[base + 4],
            a_slice[base + 5],
            a_slice[base + 6],
            a_slice[base + 7],
        ]);
        let ob = Octonion::from_array([
            b_slice[base],
            b_slice[base + 1],
            b_slice[base + 2],
            b_slice[base + 3],
            b_slice[base + 4],
            b_slice[base + 5],
            b_slice[base + 6],
            b_slice[base + 7],
        ]);
        let arr = (oa * ob).to_array();
        out.extend_from_slice(&arr);
    }

    let view = Float32Array::view(&out);
    let result = Float32Array::new_with_length(out_len as u32);
    result.set(&view, 0);
    result
}

#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = "octonionNormBatch")]
pub fn octonion_norm_batch(a: &Float32Array) -> Float32Array {
    let a_slice = a.to_vec();
    let n = a_slice.len() / 8;

    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let base = i * 8;
        let oa = Octonion::from_array([
            a_slice[base],
            a_slice[base + 1],
            a_slice[base + 2],
            a_slice[base + 3],
            a_slice[base + 4],
            a_slice[base + 5],
            a_slice[base + 6],
            a_slice[base + 7],
        ]);
        out.push(oa.norm());
    }

    let view = Float32Array::view(&out);
    let result = Float32Array::new_with_length(n as u32);
    result.set(&view, 0);
    result
}
