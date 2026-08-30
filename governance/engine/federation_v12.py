# federation_v12.py — Defederação automática com persistência

import json
import time
import threading
import requests
from datetime import datetime, timedelta
from dataclasses import dataclass, field
from typing import Dict, List, Optional
import math

@dataclass
class PeerTrustV12:
    """Métrica de confiança com persistência."""
    peer_name: str
    endpoint: str
    trust_score: float = 0.5
    success_count: int = 0
    failure_count: int = 0
    last_interaction: datetime = field(default_factory=datetime.now)
    interaction_history: List[Dict] = field(default_factory=list)
    is_active: bool = True
    persistence_id: Optional[str] = None

    def update(self, success: bool, details: Dict = None):
        self.last_interaction = datetime.now()
        self.interaction_history.append({
            "timestamp": self.last_interaction.isoformat(),
            "success": success,
            "details": details or {}
        })
        if success:
            self.success_count += 1
        else:
            self.failure_count += 1

        # Calcula trust score (mesmo algoritmo da v11)
        total = self.success_count + self.failure_count
        if total == 0:
            return

        success_rate = self.success_count / total
        recent_weight = 0.0
        if self.interaction_history:
            now = datetime.now()
            recent_entries = self.interaction_history[-10:]
            for entry in recent_entries:
                age = (now - datetime.fromisoformat(entry["timestamp"])).total_seconds()
                recency = math.exp(-age / 3600)
                recent_weight += recency * (1 if entry["success"] else 0)
            recent_weight = recent_weight / len(recent_entries) if recent_entries else 0

        consistency = 1.0
        if len(self.interaction_history) >= 5:
            recent_results = [1 if e["success"] else 0 for e in self.interaction_history[-5:]]
            mean = sum(recent_results) / len(recent_results)
            variance = sum((r - mean) ** 2 for r in recent_results) / len(recent_results)
            consistency = 1.0 - min(variance * 2, 1.0)

        self.trust_score = (
            success_rate * 0.6 +
            recent_weight * 0.3 +
            consistency * 0.1
        )

        inactivity = (datetime.now() - self.last_interaction).total_seconds()
        if inactivity > 86400:
            self.trust_score *= max(0.5, 1.0 - (inactivity / 86400) * 0.5)

        self.is_active = self.trust_score >= 0.3 and self.failure_count < 10

class FederationManagerV12:
    """
    Gerenciador de federação com trust scores persistentes.
    """

    _peers: Dict[str, PeerTrustV12] = {}
    _trust_threshold: float = 0.3
    _monitor_interval: int = 60  # segundos
    _persistence_endpoint: Optional[str] = None

    @classmethod
    def register_peer(cls, name: str, endpoint: str, initial_trust: float = 0.5):
        """Registra peer e tenta carregar estado persistente."""
        # Tenta carregar do WormGraph
        persisted = cls._load_from_ledger(name)
        if persisted:
            peer = PeerTrustV12(
                peer_name=name,
                endpoint=endpoint,
                trust_score=persisted.get("trust_score", initial_trust),
                success_count=persisted.get("success_count", 0),
                failure_count=persisted.get("failure_count", 0),
                last_interaction=datetime.fromisoformat(persisted.get("last_interaction", datetime.now().isoformat())),
                interaction_history=persisted.get("history", []),
                persistence_id=persisted.get("id")
            )
        else:
            peer = PeerTrustV12(peer_name=name, endpoint=endpoint, trust_score=initial_trust)

        cls._peers[name] = peer
        cls._persist_peer(peer)
        return peer

    @classmethod
    def evaluate_peer(cls, name: str) -> PeerTrustV12:
        """Avalia peer e atualiza trust score."""
        peer = cls._peers.get(name)
        if not peer:
            return None

        try:
            response = requests.get(f"{peer.endpoint}/health", timeout=5)
            success = response.status_code == 200
            peer.update(success, {"health_check": response.status_code})
        except Exception as e:
            peer.update(False, {"error": str(e)})

        cls._persist_peer(peer)
        return peer

    @classmethod
    def evaluate_all_peers(cls):
        """Avalia todos os peers periodicamente."""
        for name in list(cls._peers.keys()):
            cls.evaluate_peer(name)

        # Defederar peers com baixa confiança
        cls.defederate_low_trust_peers()

    @classmethod
    def defederate_low_trust_peers(cls):
        """Remove peers com confiança abaixo do threshold."""
        removed = []
        for name, peer in list(cls._peers.items()):
            if not peer.is_active or peer.trust_score < cls._trust_threshold:
                del cls._peers[name]
                removed.append(name)
                # Registra defederação no ledger
                cls._log_defederation(name, peer.trust_score)
        return removed

    @classmethod
    def _persist_peer(cls, peer: PeerTrustV12):
        """Persiste estado do peer no WormGraph."""
        if not cls._persistence_endpoint:
            return
        try:
            data = {
                "peer": peer.peer_name,
                "trust_score": peer.trust_score,
                "success_count": peer.success_count,
                "failure_count": peer.failure_count,
                "last_interaction": peer.last_interaction.isoformat(),
                "history": peer.interaction_history[-20:],  # últimos 20
                "id": peer.persistence_id
            }
            requests.post(f"{cls._persistence_endpoint}/persist_peer", json=data)
        except Exception as e:
            print(f"Failed to persist peer: {e}")

    @classmethod
    def _load_from_ledger(cls, name: str) -> Optional[Dict]:
        """Carrega estado persistido do WormGraph."""
        if not cls._persistence_endpoint:
            return None
        try:
            response = requests.get(f"{cls._persistence_endpoint}/load_peer/{name}")
            if response.status_code == 200:
                return response.json()
        except:
            pass
        return None

    @classmethod
    def _log_defederation(cls, name: str, trust_score: float):
        """Registra evento de defederação."""
        print(f"[DEFEDERATION] Removed peer {name} (trust: {trust_score:.2f})")
        if cls._persistence_endpoint:
            try:
                requests.post(f"{cls._persistence_endpoint}/log_defederation",
                              json={"peer": name, "trust": trust_score, "timestamp": datetime.now().isoformat()})
            except:
                pass

    @classmethod
    def start_monitor(cls, interval: int = 60):
        """Inicia thread de monitoramento."""
        cls._monitor_interval = interval
        def monitor():
            while True:
                time.sleep(interval)
                cls.evaluate_all_peers()
        thread = threading.Thread(target=monitor, daemon=True)
        thread.start()
        print(f"[Federation] Monitor started (interval: {interval}s)")

    @classmethod
    def set_persistence(cls, endpoint: str):
        """Define endpoint para persistência."""
        cls._persistence_endpoint = endpoint