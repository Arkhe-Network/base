# redis_cache_production_v15.py — Redis Cluster com Sentinel

import redis
from redis.sentinel import Sentinel
import os
import json
from typing import Optional, Dict, Any
import logging

class RedisCacheProduction:
    """
    Redis cache para produção com:
    - Sentinel para failover automático
    - Connection pooling
    - Serialização otimizada
    - Cache stampede protection
    """

    def __init__(self):
        self.password = os.environ.get("REDIS_PASSWORD", "")
        self.sentinel_hosts = [
            ("redis-catedral-sentinel", 26379),
        ]

        self.sentinel = Sentinel(
            self.sentinel_hosts,
            password=self.password,
            socket_timeout=5,
            retry_on_timeout=True
        )

        self._stats = {"hits": 0, "misses": 0, "errors": 0}

    def _get_client(self) -> redis.Redis:
        """Obtém o master via Sentinel"""
        return self.sentinel.master_for(
            "mymaster",
            password=self.password,
            socket_timeout=5,
            retry_on_timeout=True
        )

    def get(self, key: str) -> Optional[Any]:
        """Obtém valor do cache"""
        try:
            client = self._get_client()
            data = client.get(key)
            if data:
                self._stats["hits"] += 1
                return json.loads(data)
            self._stats["misses"] += 1
            return None
        except Exception as e:
            self._stats["errors"] += 1
            logging.error(f"Redis get error: {e}")
            return None

    def set(self, key: str, value: Any, ttl: int = 300) -> bool:
        """Armazena valor com TTL"""
        try:
            client = self._get_client()
            return client.setex(key, ttl, json.dumps(value, default=str))
        except Exception as e:
            self._stats["errors"] += 1
            logging.error(f"Redis set error: {e}")
            return False

    def setnx(self, key: str, value: Any, ttl: int = 300) -> bool:
        """Set if not exists — para cache stampede protection"""
        try:
            client = self._get_client()
            return client.set(key, json.dumps(value, default=str), ex=ttl, nx=True)
        except Exception as e:
            return False

    def get_stats(self) -> Dict:
        """Estatísticas do cache"""
        total = self._stats["hits"] + self._stats["misses"]
        return {
            "hits": self._stats["hits"],
            "misses": self._stats["misses"],
            "hit_rate": self._stats["hits"] / max(total, 1),
            "errors": self._stats["errors"]
        }