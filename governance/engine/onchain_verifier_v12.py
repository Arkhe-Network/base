# onchain_verifier_v12.py — Integração com blockchain

from web3 import Web3
from eth_account import Account
import json
from typing import Dict, List

class OnChainVerifier:
    """Verifica provas ZK em contratos inteligentes."""

    def __init__(self, rpc_url: str, contract_address: str, abi_path: str):
        self.w3 = Web3(Web3.HTTPProvider(rpc_url))
        with open(abi_path) as f:
            self.abi = json.load(f)
        self.contract = self.w3.eth.contract(address=contract_address, abi=self.abi)

    def verify(self, proof: Dict, public_signals: List[int]) -> bool:
        """
        Verifica a prova on-chain.
        """
        # Formata a prova para o contrato
        a = proof.get("a")
        b = proof.get("b")
        c = proof.get("c")
        inputs = public_signals

        # Chama o contrato
        tx_hash = self.contract.functions.verifyGovernance(a, b, c, inputs).transact()
        receipt = self.w3.eth.wait_for_transaction_receipt(tx_hash)

        # Verifica o evento
        logs = self.contract.events.Verified().process_receipt(receipt)
        return logs and logs[0].args.success