# did_manager_v12.py — DID com resolução descentralizada e cache

import json
import hashlib
import requests
from typing import Dict, Optional, List
from datetime import datetime, timedelta
import threading
import time

try:
    import didkit
    from didkit import generate_ed25519_key, key_to_did, issue_credential
    DIDKIT_AVAILABLE = True
except ImportError:
    DIDKIT_AVAILABLE = False
    print("⚠️ didkit não disponível. Use: pip install didkit")

class DIDManagerV12:
    """
    Gerenciador de DIDs W3C com resolução descentralizada, cache e suporte a múltiplos métodos.
    """

    _cache: Dict[str, Dict] = {}
    _cache_ttl: int = 300  # 5 minutos
    _agent_did_map: Dict[str, str] = {}

    @classmethod
    def generate_did(cls, method: str = "key", params: Optional[Dict] = None) -> Dict:
        """
        Gera um DID usando o método especificado.
        """
        if method == "key":
            key = generate_ed25519_key()
            did = key_to_did("key", key)
            document = cls._build_did_document(did, key["public_key"])
            cls._store_did(did, document, key["private_key"])
            return {"did": did, "document": document, "private_key": key["private_key"]}

        elif method == "web":
            # did:web requer registro em um domínio
            domain = params.get("domain", "localhost")
            did = f"did:web:{domain}"
            # Cria documento básico
            document = {
                "@context": "https://www.w3.org/ns/did/v1",
                "id": did,
                "verification_method": [{
                    "id": f"{did}#key-1",
                    "type": "Ed25519VerificationKey2020",
                    "controller": did,
                    "publicKeyMultibase": "z..."  # Seria gerado
                }]
            }
            cls._store_did(did, document, None)
            return {"did": did, "document": document}

        else:
            raise ValueError(f"Método DID {method} não suportado")

    @classmethod
    def resolve_did(cls, did: str, force: bool = False) -> Optional[Dict]:
        """
        Resolve um DID com cache e fallback para resolução descentralizada.
        """
        # Verifica cache
        if not force and did in cls._cache:
            entry = cls._cache[did]
            if datetime.now() < entry["expires"]:
                return entry["document"]

        # Tenta resolução via didkit
        if DIDKIT_AVAILABLE:
            try:
                document = didkit.resolve_did(did)
                cls._cache[did] = {
                    "document": document,
                    "expires": datetime.now() + timedelta(seconds=cls._cache_ttl)
                }
                return document
            except Exception:
                pass

        # Fallback para did:web
        if did.startswith("did:web:"):
            domain = did.replace("did:web:", "")
            url = f"https://{domain}/.well-known/did.json"
            try:
                response = requests.get(url, timeout=5)
                if response.status_code == 200:
                    document = response.json()
                    cls._cache[did] = {
                        "document": document,
                        "expires": datetime.now() + timedelta(seconds=cls._cache_ttl)
                    }
                    return document
            except:
                pass

        # Fallback para did:key
        if did.startswith("did:key:"):
            # Constrói documento a partir da chave pública (simplificado)
            public_key = "z" + did.split(":")[-1][:20]  # simulação
            document = cls._build_did_document(did, public_key)
            return document

        return None

    @classmethod
    def verify_signature(cls, did: str, message: str, signature: str) -> bool:
        """
        Verifica assinatura usando a chave pública do DID.
        """
        document = cls.resolve_did(did)
        if not document:
            return False

        # Extrai chave pública
        pub_key = document.get("verification_method", [{}])[0].get("publicKeyMultibase")
        if not pub_key:
            return False

        if DIDKIT_AVAILABLE:
            # Usa didkit para verificar
            try:
                return didkit.verify_signature(message, signature, pub_key)
            except:
                pass

        # Fallback: verificação SHA-256 (não segura, apenas para demo)
        expected = hashlib.sha256(f"{message}:{pub_key}".encode()).hexdigest()
        return signature == expected

    @classmethod
    def _build_did_document(cls, did: str, public_key: str) -> Dict:
        return {
            "@context": "https://www.w3.org/ns/did/v1",
            "id": did,
            "verification_method": [{
                "id": f"{did}#key-1",
                "type": "Ed25519VerificationKey2020",
                "controller": did,
                "publicKeyMultibase": public_key
            }],
            "authentication": [f"{did}#key-1"],
            "assertionMethod": [f"{did}#key-1"]
        }

    @classmethod
    def _store_did(cls, did: str, document: Dict, private_key: Optional[str]):
        cls._cache[did] = {
            "document": document,
            "expires": datetime.now() + timedelta(seconds=cls._cache_ttl),
            "private_key": private_key
        }