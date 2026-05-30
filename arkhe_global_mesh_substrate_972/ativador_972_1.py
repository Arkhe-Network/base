#!/usr/bin/env python3
"""
Ativador 972.1 – Inicializa NOSTR+TOR+IPFS em um nó ARKHE.
"""
import asyncio
from nostr.nostr_relay import CathedralNostrRelay
from tor.tor_service import TorMeshNode
from ipfs.ipfs_backbone import IPFSBackbone
from bridge_nostr_tor_ipfs import NostrTorIpfsBridge

async def main():
    node_id = "972.1-live-01"
    # Chaves reais viriam do keystore
    pubkey = "a"*64
    privkey = "b"*64

    bridge = NostrTorIpfsBridge(node_id, pubkey, privkey)

    print("[972.1] Iniciando bootstrap da tríade...")
    result = await bridge.bootstrap()

    print(f"\n✅ Tríade ativa:")
    print(f"  NOSTR pubkey: {result['nostr_pubkey']}")
    print(f"  TOR onion:    {result['tor_onion']}")
    print(f"  IPFS CID:     {result['ipfs_cid']}")
    print(f"  SEAL:         {result['seal']}")

    # Mantém os serviços rodando (em produção: loop infinito)
    await asyncio.sleep(10)

if __name__ == "__main__":
    asyncio.run(main())
