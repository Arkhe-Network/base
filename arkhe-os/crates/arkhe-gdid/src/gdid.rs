use alloc::format;
use alloc::string::String;
use blake3::Hasher;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Namespace {
    ArkheGlobal = 0x00,
    HubbleNetwork = 0x01,
    Oem = 0x02,
    Unknown(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Zeroize, ZeroizeOnDrop)]
pub struct Gdid(pub(crate) [u8; 32]);

impl Gdid {
    pub fn from_fingerprint(fingerprint: &[u8; 32], namespace: Namespace, nonce: u32) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(b"arkhe-gdid-v1");
        hasher.update(fingerprint);
        let hash = hasher.finalize();

        let mut bytes = [0u8; 32];
        bytes[0] = 0x01; // Version
        bytes[1] = match namespace {
            Namespace::ArkheGlobal => 0x00,
            Namespace::HubbleNetwork => 0x01,
            Namespace::Oem => 0x02,
            Namespace::Unknown(n) => n,
        };
        bytes[2..22].copy_from_slice(&hash.as_bytes()[0..20]);
        bytes[22..26].copy_from_slice(&nonce.to_le_bytes());

        Self(bytes)
    }

    pub fn version(&self) -> u8 {
        self.0[0]
    }

    pub fn namespace(&self) -> Namespace {
        match self.0[1] {
            0x00 => Namespace::ArkheGlobal,
            0x01 => Namespace::HubbleNetwork,
            0x02 => Namespace::Oem,
            n => Namespace::Unknown(n),
        }
    }

    pub fn nonce(&self) -> u32 {
        let mut nonce_bytes = [0u8; 4];
        nonce_bytes.copy_from_slice(&self.0[22..26]);
        u32::from_le_bytes(nonce_bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn raw_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_raw(bytes: &[u8; 32]) -> Self {
        Self(bytes.clone())
    }

    pub fn to_base58_raw(&self) -> String {
        let payload_b58 = bs58::encode(&self.0[..30]).into_string();
        let checksum = blake3::hash(&self.0[..30]);
        let checksum_b58 = bs58::encode(&checksum.as_bytes()[..4]).into_string();

        let ns_str = match self.namespace() {
            Namespace::ArkheGlobal => "arkhe",
            Namespace::HubbleNetwork => "hubble",
            Namespace::Oem => "oem",
            Namespace::Unknown(n) => {
                return format!("?:0x{:02x}:{}:{}", n, payload_b58, checksum_b58);
            }
        };

        format!("{}:{}:{}:{}", self.version(), ns_str, payload_b58, checksum_b58)
    }

    pub fn to_base58(&self) -> String {
        format!("gdid:{}", self.to_base58_raw())
    }

    pub fn derive(
        pubkey: &[u8; 32],
        namespace: Namespace,
        nonce: u32,
    ) -> Result<Self, crate::protocol::ProtocolError> {
        Ok(Self::from_fingerprint(pubkey, namespace, nonce))
    }
}
