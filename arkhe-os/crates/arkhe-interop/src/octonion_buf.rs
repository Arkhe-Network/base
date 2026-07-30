use crate::Octonion;

pub fn octonions_to_flat_f32(octs: &[Octonion<f32>]) -> std::vec::Vec<f32> {
    let mut buf = std::vec::Vec::with_capacity(octs.len() * 8);
    for o in octs {
        buf.extend_from_slice(&o.to_array());
    }
    buf
}

pub fn flat_to_octonions_f32(
    flat: &[f32],
) -> Result<std::vec::Vec<Octonion<f32>>, std::string::String> {
    if flat.len() % 8 != 0 {
        return Err(format!("flat buffer length {} is not a multiple of 8", flat.len()));
    }
    let n = flat.len() / 8;
    let mut out = std::vec::Vec::with_capacity(n);
    for i in 0..n {
        let base = i * 8;
        out.push(Octonion::from_array([
            flat[base],
            flat[base + 1],
            flat[base + 2],
            flat[base + 3],
            flat[base + 4],
            flat[base + 5],
            flat[base + 6],
            flat[base + 7],
        ]));
    }
    Ok(out)
}

pub fn octonions_to_flat_f64(octs: &[Octonion<f64>]) -> std::vec::Vec<f64> {
    let mut buf = std::vec::Vec::with_capacity(octs.len() * 8);
    for o in octs {
        buf.extend_from_slice(&o.to_array());
    }
    buf
}

pub fn flat_to_octonions_f64(
    flat: &[f64],
) -> Result<std::vec::Vec<Octonion<f64>>, std::string::String> {
    if flat.len() % 8 != 0 {
        return Err(format!("flat buffer length {} is not a multiple of 8", flat.len()));
    }
    let n = flat.len() / 8;
    let mut out = std::vec::Vec::with_capacity(n);
    for i in 0..n {
        let base = i * 8;
        out.push(Octonion::from_array([
            flat[base],
            flat[base + 1],
            flat[base + 2],
            flat[base + 3],
            flat[base + 4],
            flat[base + 5],
            flat[base + 6],
            flat[base + 7],
        ]));
    }
    Ok(out)
}

pub fn real_components_f32(octs: &[Octonion<f32>]) -> std::vec::Vec<f32> {
    octs.iter().map(|o| o.r).collect()
}

pub fn norm_components_f32(octs: &[Octonion<f32>]) -> std::vec::Vec<f32> {
    octs.iter().map(|o| o.norm()).collect()
}
