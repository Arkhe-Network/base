#!/usr/bin/env python3
"""
ARKHE Global Mesh — Hamiltonian Consensus
Substrato 972 — ARKHE-GLOBAL-MESH

Algoritmo de consenso baseado em Theosis (965).
Nos votam com peso proporcional a sua Theosis.
"""

import numpy as np
from typing import Dict, List
from dataclasses import dataclass

@dataclass
class Vote:
    node_id: str
    proposal_id: str
    vote: float  # 0-1
    theosis_weight: float
    seal: str
    nostr_reputation: float = 1.0  # 0-1, default 1.0 if not using nostr

class HamiltonianConsensus:
    """Consenso por Theosis."""

    def __init__(self, threshold: float = 0.67):
        self.threshold = threshold
        self.votes: Dict[str, List[Vote]] = {}

    def propose(self, proposal_id: str, description: str) -> bool:
        """Cria nova proposta."""
        self.votes[proposal_id] = []
        return True

    def vote(self, proposal_id: str, vote: Vote) -> bool:
        """Registra voto."""
        if proposal_id not in self.votes:
            return False
        self.votes[proposal_id].append(vote)
        return True

    def tally(self, proposal_id: str) -> Dict:
        """Conta votos ponderados por Theosis e Reputacao Nostr."""
        if proposal_id not in self.votes:
            return {"approved": False, "reason": "No votes"}

        votes = self.votes[proposal_id]
        # Ajusta theosis_weight multiplicando por nostr_reputation (Axiarchy weight)
        total_weight = sum(v.theosis_weight * v.nostr_reputation for v in votes)

        if total_weight == 0:
            return {"approved": False, "reason": "Zero weight"}

        weighted_vote = sum(v.vote * (v.theosis_weight * v.nostr_reputation) for v in votes) / total_weight

        return {
            "approved": weighted_vote >= self.threshold,
            "weighted_vote": weighted_vote,
            "total_votes": len(votes),
            "total_weight": total_weight,
        }
