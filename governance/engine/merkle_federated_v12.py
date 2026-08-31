# merkle_federated_v12.py — Merkle Tree federada com sincronização

import hashlib
import json
import requests
from typing import List, Dict, Optional, Tuple
from datetime import datetime
import threading
import time

class MerkleNodeV12:
    """Nó da árvore de Merkle."""
    def __init__(self, hash_value: str, left: 'MerkleNodeV12' = None, right: 'MerkleNodeV12' = None):
        self.hash = hash_value
        self.left = left
        self.right = right
        self.parent = None
        if left:
            left.parent = self
        if right:
            right.parent = self

    def is_leaf(self) -> bool:
        return self.left is None and self.right is None

class FederatedMerkleTreeV12:
    """
    Árvore de Merkle federada com sincronização pull e provas de inclusão.
    """

    def __init__(self):
        self.leaves: List[str] = []  # Hashes dos blocos
        self.root: Optional[str] = None
        self.federated_roots: Dict[str, str] = {}  # peer -> root_hash
        self.last_sync: Optional[datetime] = None
        self.lock = threading.Lock()

    @staticmethod
    def hash_data(data: any) -> str:
        if isinstance(data, dict):
            data = json.dumps(data, sort_keys=True)
        elif not isinstance(data, str):
            data = str(data)
        return hashlib.sha256(data.encode()).hexdigest()

    def add_block(self, block_data: Dict) -> str:
        """Adiciona um bloco e reconstrói a árvore."""
        with self.lock:
            block_hash = self.hash_data(block_data)
            self.leaves.append(block_hash)
            self._rebuild_tree()
            return block_hash

    def _rebuild_tree(self):
        """Reconstrói a árvore bottom-up."""
        if not self.leaves:
            self.root = None
            return

        nodes = [MerkleNodeV12(leaf) for leaf in self.leaves]
        while len(nodes) > 1:
            next_level = []
            for i in range(0, len(nodes), 2):
                if i + 1 < len(nodes):
                    combined = self.hash_data(nodes[i].hash + nodes[i+1].hash)
                    next_level.append(MerkleNodeV12(combined, nodes[i], nodes[i+1]))
                else:
                    combined = self.hash_data(nodes[i].hash + nodes[i].hash)
                    next_level.append(MerkleNodeV12(combined, nodes[i], nodes[i]))
            nodes = next_level

        self.root = nodes[0].hash

    def get_proof(self, index: int) -> List[Tuple[str, bool]]:
        """
        Gera prova de inclusão para a folha no índice index.
        Retorna lista de (hash_sibling, is_left).
        """
        if index >= len(self.leaves):
            return []

        proof = []
        # Reconstrói o caminho (implementação simplificada)
        # Em produção, manteríamos a árvore completa para gerar proof eficientemente.
        # Para este exemplo, geramos um proof simulado.
        for i in range(len(self.leaves)):
            if i == index:
                continue
            proof.append((self.leaves[i], i < index))
        return proof

    @staticmethod
    def verify_proof(leaf_hash: str, proof: List[Tuple[str, bool]], root: str) -> bool:
        """Verifica a prova de inclusão."""
        current = leaf_hash
        for sibling, is_left in proof:
            if is_left:
                current = FederatedMerkleTreeV12.hash_data(sibling + current)
            else:
                current = FederatedMerkleTreeV12.hash_data(current + sibling)
        return current == root

    def sync_with_peer(self, peer_endpoint: str) -> Dict:
        """
        Sincroniza com um peer usando pull baseado em raiz.
        """
        try:
            # Obtém a raiz do peer
            response = requests.get(f"{peer_endpoint}/merkle/root", timeout=10)
            peer_root = response.json().get("root")

            if not peer_root:
                return {"synced": False, "error": "No root returned"}

            # Se as raízes diferem, pede diferenças
            if peer_root != self.root:
                diff_response = requests.post(
                    f"{peer_endpoint}/merkle/diff",
                    json={"local_root": self.root, "local_leaves": self.leaves},
                    timeout=30
                )
                missing_blocks = diff_response.json().get("missing_blocks", [])
                for block in missing_blocks:
                    self.add_block(block)

                self.federated_roots[peer_endpoint] = peer_root
                self.last_sync = datetime.now()

                return {
                    "synced": True,
                    "blocks_added": len(missing_blocks),
                    "new_root": self.root,
                    "peer_root": peer_root
                }
            else:
                # Já sincronizado
                return {"synced": True, "blocks_added": 0}

        except Exception as e:
            return {"synced": False, "error": str(e)}

    def sync_all_peers(self, peers: List[str]) -> Dict:
        """Sincroniza com todos os peers."""
        results = {}
        for peer in peers:
            results[peer] = self.sync_with_peer(peer)
        return results

    def get_state(self) -> Dict:
        """Retorna o estado atual."""
        return {
            "leaf_count": len(self.leaves),
            "root": self.root,
            "federated_roots": self.federated_roots,
            "last_sync": self.last_sync.isoformat() if self.last_sync else None
        }