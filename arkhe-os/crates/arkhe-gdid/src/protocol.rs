use crate::gdid::{Gdid, Namespace};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GdidMessageType {
    Announce = 0x01,
    CertRequest = 0x02,
    CertResponse = 0x03,
    CrlUpdate = 0x04,
    Challenge = 0x05,
    ChallengeResponse = 0x06,
}

impl GdidMessageType {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Announce),
            0x02 => Some(Self::CertRequest),
            0x03 => Some(Self::CertResponse),
            0x04 => Some(Self::CrlUpdate),
            0x05 => Some(Self::Challenge),
            0x06 => Some(Self::ChallengeResponse),
            _ => None,
        }
    }

    pub fn topic_name(self) -> &'static str {
        match self {
            Self::Announce => "announce",
            Self::CertRequest => "cert_req",
            Self::CertResponse => "cert_resp",
            Self::CrlUpdate => "crl_update",
            Self::Challenge => "challenge",
            Self::ChallengeResponse => "challenge_resp",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    BufferTooSmall { needed: usize, got: usize },
    UnknownType(u8),
    PayloadSizeMismatch { expected: usize, got: usize },
}

impl core::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BufferTooSmall { needed, got } => {
                write!(f, "buffer too small: need {} got {}", needed, got)
            }
            Self::UnknownType(t) => write!(f, "unknown type: 0x{:02x}", t),
            Self::PayloadSizeMismatch { expected, got } => {
                write!(f, "payload size: expected {} got {}", expected, got)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ProtocolError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GdidMessage {
    pub msg_type: GdidMessageType,
    pub gdid: Gdid,
    pub payload: Vec<u8>,
}

impl GdidMessage {
    pub const HEADER_SIZE: usize = 33;

    pub fn announce(gdid: Gdid) -> Self {
        Self { msg_type: GdidMessageType::Announce, gdid, payload: Vec::new() }
    }

    pub fn cert_request(gdid: Gdid, pubkey: &[u8; 32]) -> Self {
        Self { msg_type: GdidMessageType::CertRequest, gdid, payload: pubkey.to_vec() }
    }

    pub fn challenge(gdid: Gdid, nonce: &[u8; 16]) -> Self {
        Self { msg_type: GdidMessageType::Challenge, gdid, payload: nonce.to_vec() }
    }

    pub fn encode_to_slice(&self, buf: &mut [u8]) -> Result<usize, ProtocolError> {
        let total = Self::HEADER_SIZE + self.payload.len();
        if buf.len() < total {
            return Err(ProtocolError::BufferTooSmall { needed: total, got: buf.len() });
        }
        buf[0] = self.msg_type as u8;
        buf[1..33].copy_from_slice(self.gdid.as_bytes());
        buf[33..total].copy_from_slice(&self.payload);
        Ok(total)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = vec![0u8; Self::HEADER_SIZE + self.payload.len()];
        self.encode_to_slice(&mut buf).unwrap();
        buf
    }

    pub fn decode(buf: &[u8]) -> Result<Self, ProtocolError> {
        if buf.len() < Self::HEADER_SIZE {
            return Err(ProtocolError::BufferTooSmall {
                needed: Self::HEADER_SIZE,
                got: buf.len(),
            });
        }

        let msg_type =
            GdidMessageType::from_byte(buf[0]).ok_or(ProtocolError::UnknownType(buf[0]))?;

        let mut gdid_bytes = [0u8; 32];
        gdid_bytes.copy_from_slice(&buf[1..33]);
        let gdid = Gdid::from_raw(&gdid_bytes);

        let payload = &buf[33..];
        match msg_type {
            GdidMessageType::Announce if !payload.is_empty() => {
                Err(ProtocolError::PayloadSizeMismatch { expected: 0, got: payload.len() })
            }
            GdidMessageType::CertRequest if payload.len() != 32 => {
                Err(ProtocolError::PayloadSizeMismatch { expected: 32, got: payload.len() })
            }
            GdidMessageType::Challenge if payload.len() != 16 => {
                Err(ProtocolError::PayloadSizeMismatch { expected: 16, got: payload.len() })
            }
            GdidMessageType::ChallengeResponse if payload.len() != 64 => {
                Err(ProtocolError::PayloadSizeMismatch { expected: 64, got: payload.len() })
            }
            GdidMessageType::CertResponse | GdidMessageType::CrlUpdate => {
                Ok(Self { msg_type, gdid, payload: payload.to_vec() })
            }
            _ => Ok(Self { msg_type, gdid, payload: payload.to_vec() }),
        }
    }

    pub fn pubkey(&self) -> Option<[u8; 32]> {
        if self.msg_type != GdidMessageType::CertRequest || self.payload.len() != 32 {
            return None;
        }
        let mut pk = [0u8; 32];
        pk.copy_from_slice(&self.payload);
        Some(pk)
    }

    pub fn nonce(&self) -> Option<[u8; 16]> {
        if self.msg_type != GdidMessageType::Challenge || self.payload.len() != 16 {
            return None;
        }
        let mut n = [0u8; 16];
        n.copy_from_slice(&self.payload);
        Some(n)
    }

    pub fn signature(&self) -> Option<[u8; 64]> {
        if self.msg_type != GdidMessageType::ChallengeResponse || self.payload.len() != 64 {
            return None;
        }
        let mut sig = [0u8; 64];
        sig.copy_from_slice(&self.payload);
        Some(sig)
    }
}

pub fn mqtt_topic(namespace: Namespace, msg_type: GdidMessageType) -> String {
    let ns_str = match namespace {
        Namespace::ArkheGlobal => "arkhe",
        Namespace::HubbleNetwork => "hubble",
        Namespace::Oem => "oem",
        Namespace::Unknown(n) => {
            return format!("arkhe/gdid/0x{:02x}/{}", n, msg_type.topic_name());
        }
    };
    format!("arkhe/gdid/{}/{}", ns_str, msg_type.topic_name())
}

pub fn parse_mqtt_topic(topic: &str) -> Option<(Namespace, GdidMessageType)> {
    let parts: Vec<&str> = topic.split('/').collect();
    if parts.len() != 4 || parts[0] != "arkhe" || parts[1] != "gdid" {
        return None;
    }

    let ns = match parts[2] {
        "arkhe" => Namespace::ArkheGlobal,
        "hubble" => Namespace::HubbleNetwork,
        "oem" => Namespace::Oem,
        _ => return None,
    };

    let msg_type = if let Some(mt) = GdidMessageType::from_byte(parts[3].parse().ok()?) {
        mt
    } else {
        match parts[3] {
            "announce" => GdidMessageType::Announce,
            "cert_req" => GdidMessageType::CertRequest,
            "cert_resp" => GdidMessageType::CertResponse,
            "crl_update" => GdidMessageType::CrlUpdate,
            "challenge" => GdidMessageType::Challenge,
            "challenge_resp" => GdidMessageType::ChallengeResponse,
            _ => return None,
        }
    };

    Some((ns, msg_type))
}
