#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod convert;
pub mod ffi;
pub mod ndarray_compat;
pub mod wasm;

use core::fmt;
use core::ops::{Add, Div, Mul, Neg, Sub};
use num_traits::{Float, FromPrimitive, Zero};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Zeroize)]
pub struct Octonion<T> {
    pub r: T,
    pub e1: T,
    pub e2: T,
    pub e3: T,
    pub e4: T,
    pub e5: T,
    pub e6: T,
    pub e7: T,
}

impl<T: Copy + Zero> Default for Octonion<T> {
    fn default() -> Self {
        Self {
            r: T::zero(),
            e1: T::zero(),
            e2: T::zero(),
            e3: T::zero(),
            e4: T::zero(),
            e5: T::zero(),
            e6: T::zero(),
            e7: T::zero(),
        }
    }
}

impl<T> Octonion<T> {
    #[inline]
    pub const fn new(r: T, e1: T, e2: T, e3: T, e4: T, e5: T, e6: T, e7: T) -> Self {
        Self { r, e1, e2, e3, e4, e5, e6, e7 }
    }

    #[inline]
    pub fn real(r: T) -> Self
    where
        T: Copy + Zero,
    {
        Self {
            r,
            e1: T::zero(),
            e2: T::zero(),
            e3: T::zero(),
            e4: T::zero(),
            e5: T::zero(),
            e6: T::zero(),
            e7: T::zero(),
        }
    }

    #[inline]
    pub fn to_array(&self) -> [T; 8]
    where
        T: Copy,
    {
        [self.r, self.e1, self.e2, self.e3, self.e4, self.e5, self.e6, self.e7]
    }

    #[inline]
    pub fn from_array(arr: [T; 8]) -> Self
    where
        T: Copy,
    {
        Self {
            r: arr[0],
            e1: arr[1],
            e2: arr[2],
            e3: arr[3],
            e4: arr[4],
            e5: arr[5],
            e6: arr[6],
            e7: arr[7],
        }
    }

    #[inline]
    pub fn conj(&self) -> Self
    where
        T: Neg<Output = T> + Copy,
    {
        Self {
            r: self.r,
            e1: -self.e1,
            e2: -self.e2,
            e3: -self.e3,
            e4: -self.e4,
            e5: -self.e5,
            e6: -self.e6,
            e7: -self.e7,
        }
    }

    #[inline]
    pub fn norm_sq(&self) -> T
    where
        T: Copy + Mul<Output = T> + Add<Output = T>,
    {
        self.r * self.r
            + self.e1 * self.e1
            + self.e2 * self.e2
            + self.e3 * self.e3
            + self.e4 * self.e4
            + self.e5 * self.e5
            + self.e6 * self.e6
            + self.e7 * self.e7
    }

    #[inline]
    pub fn norm(&self) -> T
    where
        T: Copy + Mul<Output = T> + Add<Output = T> + Float,
    {
        self.norm_sq().sqrt()
    }

    #[inline]
    pub fn inv(&self) -> Option<Self>
    where
        T: Copy
            + Neg<Output = T>
            + Mul<Output = T>
            + Add<Output = T>
            + Div<Output = T>
            + PartialEq
            + Float
            + Zero,
    {
        let n2 = self.norm_sq();
        if n2 == T::zero() {
            return None;
        }
        let conj = self.conj();
        Some(Self {
            r: conj.r / n2,
            e1: conj.e1 / n2,
            e2: conj.e2 / n2,
            e3: conj.e3 / n2,
            e4: conj.e4 / n2,
            e5: conj.e5 / n2,
            e6: conj.e6 / n2,
            e7: conj.e7 / n2,
        })
    }
}

#[inline]
fn quaternion_mul<T>(a: (T, T, T, T), b: (T, T, T, T)) -> (T, T, T, T)
where
    T: Copy + Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Neg<Output = T>,
{
    let (a1, a2, a3, a4) = a;
    let (b1, b2, b3, b4) = b;
    (
        a1 * b1 - a2 * b2 - a3 * b3 - a4 * b4,
        a1 * b2 + a2 * b1 + a3 * b4 - a4 * b3,
        a1 * b3 - a2 * b4 + a3 * b1 + a4 * b2,
        a1 * b4 + a2 * b3 - a3 * b2 + a4 * b1,
    )
}

#[inline]
fn quaternion_conj<T>(q: (T, T, T, T)) -> (T, T, T, T)
where
    T: Neg<Output = T> + Copy,
{
    (q.0, -q.1, -q.2, -q.3)
}

impl<T> Mul for Octonion<T>
where
    T: Copy + Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Neg<Output = T> + FromPrimitive,
{
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        let a = (self.r, self.e1, self.e2, self.e3);
        let b = (self.e4, self.e5, self.e6, self.e7);
        let c = (rhs.r, rhs.e1, rhs.e2, rhs.e3);
        let d = (rhs.e4, rhs.e5, rhs.e6, rhs.e7);

        let ac = quaternion_mul(a, c);
        let d_conj_b = quaternion_mul(d, quaternion_conj(b));
        let conj_a_d = quaternion_mul(quaternion_conj(a), d);
        let c_b = quaternion_mul(c, b);

        let real_q = (ac.0 - d_conj_b.0, ac.1 - d_conj_b.1, ac.2 - d_conj_b.2, ac.3 - d_conj_b.3);
        let imag_q =
            (conj_a_d.0 + c_b.0, conj_a_d.1 + c_b.1, conj_a_d.2 + c_b.2, conj_a_d.3 + c_b.3);

        Self {
            r: real_q.0,
            e1: real_q.1,
            e2: real_q.2,
            e3: real_q.3,
            e4: imag_q.0,
            e5: imag_q.1,
            e6: imag_q.2,
            e7: imag_q.3,
        }
    }
}

impl<T> Add for Octonion<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            r: self.r + rhs.r,
            e1: self.e1 + rhs.e1,
            e2: self.e2 + rhs.e2,
            e3: self.e3 + rhs.e3,
            e4: self.e4 + rhs.e4,
            e5: self.e5 + rhs.e5,
            e6: self.e6 + rhs.e6,
            e7: self.e7 + rhs.e7,
        }
    }
}

impl<T> Sub for Octonion<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            r: self.r - rhs.r,
            e1: self.e1 - rhs.e1,
            e2: self.e2 - rhs.e2,
            e3: self.e3 - rhs.e3,
            e4: self.e4 - rhs.e4,
            e5: self.e5 - rhs.e5,
            e6: self.e6 - rhs.e6,
            e7: self.e7 - rhs.e7,
        }
    }
}

impl<T> Neg for Octonion<T>
where
    T: Neg<Output = T> + Copy,
{
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            r: -self.r,
            e1: -self.e1,
            e2: -self.e2,
            e3: -self.e3,
            e4: -self.e4,
            e5: -self.e5,
            e6: -self.e6,
            e7: -self.e7,
        }
    }
}

impl<T: fmt::Display + Copy> fmt::Display for Octonion<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({} + {}e1 + {}e2 + {}e3 + {}e4 + {}e5 + {}e6 + {}e7)",
            self.r, self.e1, self.e2, self.e3, self.e4, self.e5, self.e6, self.e7
        )
    }
}

#[cfg(feature = "neural")]
pub mod neural {
    use super::*;
    use alloc::vec::Vec;

    pub struct OctonionLinear<T> {
        pub weights: Vec<Octonion<T>>,
        pub bias: Octonion<T>,
    }

    impl<T> OctonionLinear<T>
    where
        T: Copy
            + Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Neg<Output = T>
            + FromPrimitive,
    {
        pub fn forward(&self, input: &[Octonion<T>]) -> Vec<Octonion<T>> {
            input.iter().zip(self.weights.iter()).map(|(x, w)| *w * *x + self.bias).collect()
        }
    }
}
