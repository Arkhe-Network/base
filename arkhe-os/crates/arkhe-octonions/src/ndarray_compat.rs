
#[cfg(feature = "ndarray")]
pub struct OctonionArrayView<'a, T> {
    inner: ndarray::ArrayView2<'a, T>,
}

#[cfg(feature = "ndarray")]
impl<'a, T: Copy> OctonionArrayView<'a, T> {
    pub fn from_slice(octs: &'a [Octonion<T>]) -> Self {
        assert!(!octs.is_empty(), "octonion slice must not be empty");
        let n = octs.len();

        #[cfg(feature = "bytemuck")]
        let flat: &'a [T] = bytemuck::cast_slice(octs);

        #[cfg(not(feature = "bytemuck"))]
        let flat: &'a [T] = unsafe {
            let ptr = octs.as_ptr() as *const T;
            let len = n * 8;
            core::slice::from_raw_parts(ptr, len)
        };

        Self {
            inner: ndarray::ArrayView2::from_shape((n, 8), flat)
                .expect("shape (n, 8) must be valid for n*8 elements"),
        }
    }

    pub fn as_array(&self) -> ndarray::ArrayView2<'a, T> {
        self.inner
    }

    pub fn len(&self) -> usize {
        self.inner.nrows()
    }

    pub fn get(&self, idx: usize) -> Option<Octonion<T>> {
        if idx >= self.len() {
            return None;
        }
        let row = self.inner.row(idx);
        Some(Octonion {
            r: row[0],
            e1: row[1],
            e2: row[2],
            e3: row[3],
            e4: row[4],
            e5: row[5],
            e6: row[6],
            e7: row[7],
        })
    }
}

#[cfg(feature = "ndarray")]
pub struct OctonionArrayViewMut<'a, T> {
    inner: ndarray::ArrayViewMut2<'a, T>,
}

#[cfg(feature = "ndarray")]
impl<'a, T: Copy> OctonionArrayViewMut<'a, T> {
    pub fn from_slice_mut(octs: &'a mut [Octonion<T>]) -> Self {
        assert!(!octs.is_empty(), "octonion slice must not be empty");
        let n = octs.len();

        #[cfg(feature = "bytemuck")]
        let flat: &'a mut [T] = bytemuck::cast_slice_mut(octs);

        #[cfg(not(feature = "bytemuck"))]
        let flat: &'a mut [T] = unsafe {
            let ptr = octs.as_mut_ptr() as *mut T;
            let len = n * 8;
            core::slice::from_raw_parts_mut(ptr, len)
        };

        Self {
            inner: ndarray::ArrayViewMut2::from_shape((n, 8), flat)
                .expect("shape (n, 8) must be valid"),
        }
    }

    pub fn as_array_mut(&mut self) -> ndarray::ArrayViewMut2<'a, T> {
        self.inner.reborrow()
    }

    pub fn len(&self) -> usize {
        self.inner.nrows()
    }

    pub fn set(&mut self, idx: usize, val: Octonion<T>) {
        assert!(idx < self.len(), "index out of bounds");
        let mut row = self.inner.row_mut(idx);
        row[0] = val.r;
        row[1] = val.e1;
        row[2] = val.e2;
        row[3] = val.e3;
        row[4] = val.e4;
        row[5] = val.e5;
        row[6] = val.e6;
        row[7] = val.e7;
    }
}

#[cfg(feature = "ndarray")]
pub fn from_ndarray<S>(
    arr: &ndarray::ArrayBase<S, ndarray::Ix2>,
) -> alloc::vec::Vec<Octonion<S::Elem>>
where
    S: ndarray::Data,
    S::Elem: Copy,
{
    assert_eq!(
        arr.shape()[1],
        8,
        "ndarray must be (N, 8), got ({}, {})",
        arr.shape()[0],
        arr.shape()[1]
    );
    let n = arr.shape()[0];
    (0..n)
        .map(|i| {
            let row = arr.row(i);
            Octonion {
                r: row[0],
                e1: row[1],
                e2: row[2],
                e3: row[3],
                e4: row[4],
                e5: row[5],
                e6: row[6],
                e7: row[7],
            }
        })
        .collect()
}

#[cfg(feature = "ndarray")]
pub fn to_ndarray<T: Copy>(octs: &[Octonion<T>]) -> ndarray::Array2<T> {
    let n = octs.len();
    if n == 0 {
        return ndarray::Array2::from_shape_vec((0, 8), alloc::vec![]).unwrap();
    }
    let mut flat = alloc::vec::Vec::with_capacity(n * 8);
    for o in octs {
        flat.extend_from_slice(&o.to_array());
    }
    ndarray::Array2::from_shape_vec((n, 8), flat).expect("shape (n, 8) valid for n*8 elements")
}
