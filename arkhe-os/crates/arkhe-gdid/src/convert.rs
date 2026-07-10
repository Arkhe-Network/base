use crate::gdid::Gdid;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvertError {
    BufferTooSmall { needed: usize, got: usize },
    InvalidFormat(String),
    LossyConversion(String),
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferTooSmall { needed, got } => {
                write!(f, "buffer too small: need {} bytes, got {}", needed, got)
            }
            Self::InvalidFormat(msg) => write!(f, "invalid format: {}", msg),
            Self::LossyConversion(msg) => write!(f, "lossy conversion: {}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ConvertError {}

impl Gdid {
    pub fn to_hex(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let bytes = self.as_bytes();
        let mut s = String::with_capacity(64);
        for &b in bytes {
            s.push(HEX[(b >> 4) as usize] as char);
            s.push(HEX[(b & 0x0f) as usize] as char);
        }
        s
    }

    pub fn to_hex_buf(&self, buf: &mut [u8; 64]) {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for (i, &b) in self.as_bytes().iter().enumerate() {
            buf[i * 2] = HEX[(b >> 4) as usize];
            buf[i * 2 + 1] = HEX[(b & 0x0f) as usize];
        }
    }

    pub fn from_hex(s: &str) -> Result<Self, ConvertError> {
        if s.len() != 64 {
            return Err(ConvertError::InvalidFormat(format!(
                "hex must be 64 chars, got {}",
                s.len()
            )));
        }
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] = (hex_val(s.as_bytes()[i * 2])? << 4) | hex_val(s.as_bytes()[i * 2 + 1])?;
        }
        Ok(Gdid::from_raw(&bytes))
    }
}

fn hex_val(c: u8) -> Result<u8, ConvertError> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(ConvertError::InvalidFormat(format!("invalid hex char: '{}'", c as char))),
    }
}

impl Gdid {
    pub fn to_did_string(&self) -> String {
        format!("did:arkhe:gdid:{}", self.to_base58_raw())
    }

    pub fn from_did_string(s: &str) -> Result<Self, ConvertError> {
        let inner = if let Some(rest) = s.strip_prefix("did:arkhe:gdid:") {
            rest
        } else if let Some(rest) = s.strip_prefix("gdid:") {
            rest
        } else {
            return Err(ConvertError::InvalidFormat(
                "must start with 'did:arkhe:gdid:' or 'gdid:'".into(),
            ));
        };

        let parts: Vec<&str> = inner.split(':').collect();
        if parts.len() != 4 {
            return Err(ConvertError::InvalidFormat(format!(
                "expected 4 colon-separated parts after prefix, got {}",
                parts.len()
            )));
        }

        let payload_b58 = parts[2];
        let checksum_b58 = parts[3];

        let mut payload_bytes = [0u8; 30];
        let decoded_len = bs58::decode(payload_b58)
            .into_vec()
            .map_err(|e| ConvertError::InvalidFormat(format!("base58 payload: {}", e)))?;

        if decoded_len.len() != 30 {
            return Err(ConvertError::InvalidFormat(format!(
                "payload decoded to {} bytes, expected 30",
                decoded_len.len()
            )));
        }
        payload_bytes.copy_from_slice(&decoded_len);

        let mut checksum_bytes = [0u8; 4];
        let ck_len = bs58::decode(checksum_b58)
            .into_vec()
            .map_err(|e| ConvertError::InvalidFormat(format!("base58 checksum: {}", e)))?;

        if ck_len.len() != 4 {
            return Err(ConvertError::InvalidFormat(format!(
                "checksum decoded to {} bytes, expected 4",
                ck_len.len()
            )));
        }
        checksum_bytes.copy_from_slice(&ck_len);

        let expected = blake3::hash(&payload_bytes);
        if expected.as_bytes()[..4] != checksum_bytes {
            return Err(ConvertError::InvalidFormat("checksum mismatch".into()));
        }

        let mut full = [0u8; 32];
        full[..30].copy_from_slice(&payload_bytes);
        Ok(Gdid::from_raw(&full))
    }
}

impl Gdid {
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.as_bytes()
    }

    pub fn copy_to_slice(&self, buf: &mut [u8]) -> Result<(), ConvertError> {
        if buf.len() < 32 {
            return Err(ConvertError::BufferTooSmall { needed: 32, got: buf.len() });
        }
        buf[..32].copy_from_slice(self.as_bytes());
        Ok(())
    }
}
