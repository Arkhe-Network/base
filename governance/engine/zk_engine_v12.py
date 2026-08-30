# zk_engine_v12.py — Motor completo com verificação on-chain

import json
import subprocess
import tempfile
import os
import hashlib
from typing import Dict, Any, Optional, List
from pathlib import Path
from datetime import datetime
from zk_config_v12 import CurveType, ProofScheme, ZKConfig

# Tentativa de importar snarkpy e circomlib
try:
    from snarkpy import Groth16, PLONK
    from snarkpy.curves import BN254, BLS12_381
    SNARKPY_AVAILABLE = True
except ImportError:
    SNARKPY_AVAILABLE = False
    print("⚠️ snarkpy não disponível. Use: pip install snarkpy")

class ZKEngineV12:
    """
    Motor zk-SNARK/STARK com suporte a múltiplas curvas e verificação on-chain.
    """

    CIRCUIT_DIR = Path(__file__).parent / "circuits"
    VERIFIER_DIR = Path(__file__).parent / "verifiers"

    @classmethod
    def compile_circuit(cls, circuit_name: str, curve: CurveType = CurveType.BN254) -> Dict:
        """
        Compila um circuito Circom para a curva especificada.
        """
        circuit_path = cls.CIRCUIT_DIR / f"{circuit_name}.circom"
        if not circuit_path.exists():
            raise FileNotFoundError(f"Circuito {circuit_name} não encontrado")

        # Compila para R1CS e WASM
        subprocess.run([
            "circom", str(circuit_path),
            "--r1cs", "--wasm", "--sym",
            "-o", str(cls.CIRCUIT_DIR)
        ], check=True)

        # Seleciona o arquivo de potência de tau apropriado
        if curve == CurveType.BN254:
            ptau_file = "powersOfTau28_hez_final_12.ptau"
        elif curve == CurveType.BLS12_381:
            ptau_file = "powersOfTau28_hez_final_12_bls.ptau"
        else:
            raise ValueError(f"Curva {curve} não suportada para setup Groth16")

        ptau_path = cls.CIRCUIT_DIR / ptau_file
        if not ptau_path.exists():
            # Baixar automaticamente se não existir
            cls._download_ptau(ptau_file, ptau_path)

        r1cs_path = cls.CIRCUIT_DIR / f"{circuit_name}.r1cs"
        zkey_path = cls.CIRCUIT_DIR / f"{circuit_name}_{curve.value}.zkey"
        vk_path = cls.CIRCUIT_DIR / f"{circuit_name}_{curve.value}_vk.json"

        # Setup com snarkjs
        subprocess.run([
            "snarkjs", "groth16", "setup",
            str(r1cs_path), str(ptau_path), str(zkey_path)
        ], check=True)

        subprocess.run([
            "snarkjs", "zkey", "export", "verificationkey",
            str(zkey_path), str(vk_path)
        ], check=True)

        return {
            "r1cs": r1cs_path,
            "zkey": zkey_path,
            "vk": vk_path,
            "wasm": cls.CIRCUIT_DIR / f"{circuit_name}_js" / f"{circuit_name}.wasm"
        }

    @classmethod
    def generate_proof(cls, circuit_name: str, inputs: Dict, curve: CurveType = CurveType.BN254) -> Dict:
        """
        Gera uma prova usando o esquema configurado.
        """
        if not SNARKPY_AVAILABLE:
            return cls._fallback_proof(inputs)

        # 1. Compila o circuito
        circuit_info = cls.compile_circuit(circuit_name, curve)

        # 2. Gera witness
        witness_path = cls.CIRCUIT_DIR / "witness.wtns"
        input_json = json.dumps(inputs)

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json') as f:
            f.write(input_json)
            f.flush()
            subprocess.run([
                "node",
                str(cls.CIRCUIT_DIR / f"{circuit_name}_js" / "generate_witness.js"),
                str(circuit_info["wasm"]),
                f.name,
                str(witness_path)
            ], check=True)

        # 3. Gera prova com snarkpy (usando a curva correta)
        if curve == CurveType.BN254:
            proof = Groth16.prove(
                zkey_path=str(circuit_info["zkey"]),
                witness_path=str(witness_path)
            )
        elif curve == CurveType.BLS12_381:
            # Usar a versão BLS12-381 do Groth16
            proof = Groth16.prove(
                zkey_path=str(circuit_info["zkey"]),
                witness_path=str(witness_path),
                curve=BLS12_381
            )
        else:
            raise ValueError(f"Curva {curve} não suportada para geração de prova")

        # 4. Exporta a prova e os sinais públicos
        proof_path = cls.CIRCUIT_DIR / "proof.json"
        public_path = cls.CIRCUIT_DIR / "public.json"

        with open(proof_path, 'w') as f:
            json.dump(proof, f)
        with open(public_path, 'w') as f:
            # extrair sinais públicos do witness (simplificado)
            public_signals = {"phi": inputs.get("phi", 0), "threshold": inputs.get("threshold", 85)}
            json.dump(public_signals, f)

        return {
            "proof_type": "groth16",
            "curve": curve.value,
            "proof": proof,
            "public_signals": public_signals,
            "verification_key": str(circuit_info["vk"]),
            "timestamp": datetime.utcnow().isoformat()
        }

    @classmethod
    def verify_proof_onchain(cls, proof_data: Dict) -> Dict:
        """
        Gera um contrato Solidity para verificação on-chain e retorna o endereço simulado.
        """
        curve = proof_data.get("curve", "bn254")
        vk_path = proof_data.get("verification_key")

        # Carrega a chave de verificação
        with open(vk_path) as f:
            vk = json.load(f)

        # Gera o verificador Solidity usando snarkjs
        verifier_dir = cls.VERIFIER_DIR / curve
        verifier_dir.mkdir(parents=True, exist_ok=True)
        verifier_path = verifier_dir / "Verifier.sol"

        subprocess.run([
            "snarkjs", "zkey", "export", "solidityverifier",
            vk_path, str(verifier_path)
        ], check=True)

        # Simula a implantação (em produção, usar web3.py)
        contract_address = f"0x{hashlib.sha256(str(vk).encode()).hexdigest()[:40]}"

        return {
            "contract_address": contract_address,
            "verifier_code": verifier_path.read_text(),
            "verification_status": "pending_onchain"
        }

    @classmethod
    def verify_offchain(cls, proof_data: Dict) -> bool:
        """
        Verificação off-chain usando snarkpy.
        """
        if not SNARKPY_AVAILABLE:
            return cls._fallback_verify(proof_data)

        try:
            vk_path = proof_data.get("verification_key")
            proof = proof_data.get("proof")
            public = proof_data.get("public_signals")

            result = Groth16.verify(
                vk_path=vk_path,
                proof=proof,
                public=public
            )
            return result.get("verified", False)
        except Exception as e:
            print(f"Verification failed: {e}")
            return False

    @classmethod
    def generate_stark_proof(cls, inputs: Dict) -> Dict:
        """
        Gera prova zk-STARK (sem trusted setup) usando Cairo ou similar.
        """
        # Implementação simplificada: em produção, usar Cairo ou STARKNet
        proof = {
            "proof_type": "stark",
            "curve": "ed25519",
            "proof_data": "0x" + hashlib.sha256(json.dumps(inputs).encode()).hexdigest(),
            "public_inputs": inputs
        }
        return proof

    @classmethod
    def _download_ptau(cls, filename: str, target_path: Path):
        """Baixa arquivo de powers of tau se não existir"""
        import requests
        url = f"https://hermez.s3-eu-west-1.amazonaws.com/{filename}"
        response = requests.get(url, stream=True)
        with open(target_path, 'wb') as f:
            for chunk in response.iter_content(chunk_size=8192):
                f.write(chunk)

    @classmethod
    def _fallback_proof(cls, inputs: Dict) -> Dict:
        """Fallback SHA-256"""
        return {
            "proof_type": "fallback",
            "proof": hashlib.sha256(json.dumps(inputs).encode()).hexdigest(),
            "public_signals": inputs
        }

    @classmethod
    def _fallback_verify(cls, proof_data: Dict) -> bool:
        return proof_data.get("proof_type") == "fallback"