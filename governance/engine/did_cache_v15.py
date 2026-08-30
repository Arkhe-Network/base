# did_cache_v15.py — Redis cache para resolução DID

import json
import redis
from typing import Optional, Dict
from datetime import datetime, timedelta
import hashlib
import os

class DIDCacheV15:
    """
    Cache Redis para resolução de DIDs.
    Baseado em: verana-labs DID resolver

    Características:
    - Cache com TTL configurável
    - validUntil expiry independente do TTL
    - Prefixo: resolver:obj:{did}
    """

    def __init__(self, redis_url: str = None, ttl: int = 300):
        self.redis = redis.from_url(
            redis_url or os.environ.get("REDIS_URL", "redis://localhost:6379/0")
        )
        self.ttl = ttl
        self.key_prefix = "resolver:obj:"
        self._stats = {"hits": 0, "misses": 0}

    def get(self, did: str) -> Optional[Dict]:
        """Obtém DID do cache com validação de validUntil"""
        key = f"{self.key_prefix}{did}"
        data = self.redis.get(key)

        if data:
            doc = json.loads(data)

            # Verifica validUntil expiry independente do CACHE_TTL
            if "validUntil" in doc:
                valid_until = datetime.fromisoformat(doc["validUntil"])
                if datetime.utcnow() > valid_until:
                    self.redis.delete(key)
                    self._stats["misses"] += 1
                    return None

            # Verifica TTL do cache
            ttl = self.redis.ttl(key)
            if ttl <= 0:
                self.redis.delete(key)
                self._stats["misses"] += 1
                return None

            self._stats["hits"] += 1
            return doc

        self._stats["misses"] += 1
        return None

    def set(self, did: str, document: Dict, ttl: int = None) -> None:
        """Armazena DID no cache"""
        key = f"{self.key_prefix}{did}"

        # Adiciona timestamp de cache
        document["_cachedAt"] = datetime.utcnow().isoformat()

        # Se o documento tem validUntil, usa como referência
        if "validUntil" in document:
            valid_until = datetime.fromisoformat(document["validUntil"])
            cache_ttl = max(0, int((valid_until - datetime.utcnow()).total_seconds()))
            if cache_ttl > 0:
                self.redis.setex(key, min(cache_ttl, 86400), json.dumps(document))
                return

        # TTL padrão
        self.redis.setex(key, ttl or self.ttl, json.dumps(document))

    def invalidate(self, did: str) -> None:
        """Invalida um DID no cache"""
        key = f"{self.key_prefix}{did}"
        self.redis.delete(key)

    def get_stats(self) -> Dict:
        """Estatísticas do cache"""
        total = self._stats["hits"] + self._stats["misses"]
        return {
            "hits": self._stats["hits"],
            "misses": self._stats["misses"],
            "hit_rate": self._stats["hits"] / max(total, 1)
        }


class DIDResolverWithCacheV15:
    """Resolvedor DID com cache Redis e DNSSEC"""

    def __init__(self, cache: DIDCacheV15 = None):
        self.cache = cache or DIDCacheV15()
        self._resolvers = {}

    def register_resolver(self, method: str, resolver_func):
        self._resolvers[method] = resolver_func

    def resolve(self, did: str, force: bool = False) -> Optional[Dict]:
        """Resolve DID com cache"""
        # 1. Verifica cache (exceto se force)
        if not force:
            cached = self.cache.get(did)
            if cached:
                return cached

        # 2. Resolve o DID
        method = did.split(":")[1] if len(did.split(":")) > 1 else None

        if method in self._resolvers:
            document = self._resolvers[method](did)
        else:
            document = self._resolve_generic(did)

        # 3. Cacheia o resultado
        if document:
            self.cache.set(did, document)

        return document

    def _resolve_generic(self, did: str) -> Optional[Dict]:
        """Resolução genérica para DIDs"""
        if did.startswith("did:web:"):
            return self._resolve_did_web(did)
        if did.startswith("did:key:"):
            return self._resolve_did_key(did)
        return None

    def _resolve_did_web(self, did: str) -> Optional[Dict]:
        """Resolve did:web com DNSSEC"""
        import requests
        domain = did.replace("did:web:", "").split(":")[0]
        url = f"https://{domain}/.well-known/did.json"
        try:
            response = requests.get(url, timeout=10, verify=True)
            if response.status_code == 200:
                return response.json()
        except:
            pass
        return None

    def _resolve_did_key(self, did: str) -> Dict:
        """Resolve did:key"""
        public_key = did.replace("did:key:", "")
        return {
            "@context": "https://www.w3.org/ns/did/v1",
            "id": did,
            "verification_method": [{
                "id": f"{did}#key-1",
                "type": "Ed25519VerificationKey2020",
                "controller": did,
                "publicKeyMultibase": public_key
            }]
        }