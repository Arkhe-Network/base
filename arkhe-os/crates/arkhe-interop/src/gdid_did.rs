use crate::Gdid;

pub fn gdid_to_did(gdid: &Gdid) -> String {
    format!("did:arkhe:device:{}", gdid.to_base58_raw())
}

pub fn did_to_gdid(did: &str) -> Result<Gdid, String> {
    let inner = if let Some(rest) = did.strip_prefix("did:arkhe:device:") {
        rest
    } else if let Some(rest) = did.strip_prefix("did:arkhe:gdid:") {
        rest
    } else if let Some(rest) = did.strip_prefix("gdid:") {
        rest
    } else {
        return Err(format!("unrecognized DID format: {}", did));
    };

    Gdid::from_did_string(&format!("gdid:{}", inner)).map_err(|e| e.to_string())
}

#[cfg(feature = "json")]
pub fn gdid_did_document(gdid: &Gdid) -> Result<String, String> {
    let did_id = gdid_to_did(gdid);
    let key_id = format!("{}#key-1", did_id);

    let pubkey_multibase = format!("z{}", bs58::encode(gdid.as_bytes()).into_string());

    let doc = serde_json::json!({
        "@context": [
            "https://www.w3.org/ns/did/v1",
            "https://w3id.org/security/suites/ed25519-2020/v1"
        ],
        "id": did_id,
        "verificationMethod": [{
            "id": key_id,
            "type": "Ed25519VerificationKey2020",
            "controller": did_id,
            "publicKeyMultibase": pubkey_multibase,
        }],
        "authentication": [key_id],
        "assertionMethod": [key_id],
        "capabilityInvocation": [key_id],
        "capabilityDelegation": [key_id],
    });

    serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())
}
