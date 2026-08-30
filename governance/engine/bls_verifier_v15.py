# bls_verifier_v15.py — Verificação BLS12-381 on-chain

from web3 import Web3
import os
import json

class BLSVerifierOnChain:
    """Verificador BLS12-381 on-chain via EIP-2537"""

    def __init__(self, rpc_url: str = "https://rpc.sepolia.org"):
        self.w3 = Web3(Web3.HTTPProvider(rpc_url))
        self.verifier_address = os.environ.get("BLS_VERIFIER_ADDRESS")

        self.abi = [
            {
                "inputs": [
                    {"type": "bytes", "name": "a"},
                    {"type": "bytes", "name": "b"},
                    {"type": "bytes", "name": "c"},
                    {"type": "bytes32", "name": "input"}
                ],
                "name": "verifyProof",
                "outputs": [{"type": "bool"}],
                "stateMutability": "view",
                "type": "function"
            }
        ]

        self.contract = self.w3.eth.contract(
            address=self.verifier_address,
            abi=self.abi
        )

    def verify(self, proof_data: dict) -> bool:
        """Verifica prova BLS12-381 on-chain"""
        try:
            a = bytes.fromhex(proof_data["proof"]["a"])
            b = bytes.fromhex(proof_data["proof"]["b"])
            c = bytes.fromhex(proof_data["proof"]["c"])
            # Note: requires properly sized inputs
            input_val = bytes.fromhex(proof_data["public_signals"].get("phi", "00"*32))

            result = self.contract.functions.verifyProof(a, b, c, input_val).call()
            return result
        except Exception as e:
            print(f"BLS12-381 verification failed: {e}")
            return False