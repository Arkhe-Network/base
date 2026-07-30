
#[cfg(feature = "uuid")]
pub fn gdid_to_uuid(gdid: &Gdid) -> uuid::Uuid {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, gdid.as_bytes())
}

#[cfg(feature = "uuid")]
pub fn uuid_to_gdid(u: uuid::Uuid) -> Gdid {
    let u_bytes = u.as_bytes();
    let mut gdid_bytes = [0u8; 32];
    gdid_bytes[0] = 0x01;
    gdid_bytes[1] = 0x00;
    gdid_bytes[2..18].copy_from_slice(u_bytes);
    Gdid::from_raw(&gdid_bytes)
}

#[cfg(feature = "uuid")]
pub struct UuidConversionInfo {
    pub preserved_bytes: usize,
    pub zeroed_bytes: usize,
    pub nonce_preserved: bool,
}

#[cfg(feature = "uuid")]
impl UuidConversionInfo {
    pub fn for_conversion() -> Self {
        Self { preserved_bytes: 16, zeroed_bytes: 14, nonce_preserved: false }
    }
}
