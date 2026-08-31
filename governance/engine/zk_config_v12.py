# zk_config_v12.py — Suporte a múltiplas curvas e esquemas

from enum import Enum
from dataclasses import dataclass
from typing import Dict, Optional

class CurveType(Enum):
    BN254 = "bn254"           # Ethereum, Groth16
    BLS12_381 = "bls12_381"   # Filecoin, Zcash
    SECP256K1 = "secp256k1"   # Bitcoin, Ethereum (ECDSA)
    ED25519 = "ed25519"       # DID, ZK-STARKs

class ProofScheme(Enum):
    GROTH16 = "groth16"
    PLONK = "plonk"
    STARK = "stark"

@dataclass
class ZKConfig:
    curve: CurveType = CurveType.BN254
    scheme: ProofScheme = ProofScheme.GROTH16
    use_trusted_setup: bool = True
    power_of_tau: int = 12  # Para setup